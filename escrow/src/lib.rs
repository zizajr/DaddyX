//! TicketDaddy Multi-Token Escrow Program
//!
//! Production-ready escrow system for the TicketDaddy platform on Solana.
//! 
//! Features:
//! - Multi-token support (SOL + SPL Tokens)
//! - 5% platform fee (500 basis points)
//! - Business registry system
//! - Custom release times (instant, 1 hour, 48 hours, or event-based)
//! - NFT receipt integration via CPI
//! - Upgradeable via BPF loader
//!
//! Mirrors the functionality of MultiTokenEscrowUpgradeable.sol

use anchor_lang::prelude::*;
use anchor_spl::token::{self, Token, TokenAccount, Transfer as SplTransfer};
use anchor_spl::associated_token::AssociatedToken;

declare_id!("TDXEscrow1111111111111111111111111111111");

// =============================================================================
// CONSTANTS (Mirroring Solidity contract)
// =============================================================================

/// Native SOL marker (similar to 0xEeee... in EVM)
pub const NATIVE_SOL: Pubkey = Pubkey::new_from_array([0u8; 32]);

/// Platform fee: 5% = 500 basis points
pub const PLATFORM_FEE_BASIS_POINTS: u64 = 500;

/// Basis points denominator
pub const BASIS_POINTS_DENOMINATOR: u64 = 10000;

/// Minimum escrow duration: 10 minutes
pub const MIN_ESCROW_DURATION: i64 = 600;

/// Maximum escrow duration: 30 days
pub const MAX_ESCROW_DURATION: i64 = 30 * 24 * 60 * 60;

/// Maximum release time from now: 1 year
pub const MAX_RELEASE_TIME: i64 = 365 * 24 * 60 * 60;

/// Minimum native SOL amount: 0.001 SOL
pub const MIN_SOL_AMOUNT: u64 = 1_000_000; // lamports

/// Maximum business ID length
pub const MAX_BUSINESS_ID_LEN: usize = 64;

/// Maximum escrow ID length
pub const MAX_ESCROW_ID_LEN: usize = 64;

/// Maximum business name length
pub const MAX_BUSINESS_NAME_LEN: usize = 128;

/// Maximum metadata URI length
pub const MAX_METADATA_URI_LEN: usize = 256;

/// Maximum product type fee: 50% = 5000 basis points
pub const MAX_PRODUCT_TYPE_FEE: u64 = 5000;

/// Number of product types
pub const PRODUCT_TYPE_COUNT: usize = 9;

/// Maximum batch size for batch operations
pub const MAX_BATCH_SIZE: usize = 10;

// =============================================================================
// PROGRAM
// =============================================================================

#[program]
pub mod ticketdaddy_escrow {
    use super::*;

    // =========================================================================
    // INITIALIZATION
    // =========================================================================

    /// Initialize the escrow program state
    /// 
    /// Similar to Solidity's initialize() function
    pub fn initialize(
        ctx: Context<Initialize>,
        escrow_duration: i64,
    ) -> Result<()> {
        require!(
            escrow_duration >= MIN_ESCROW_DURATION && escrow_duration <= MAX_ESCROW_DURATION,
            EscrowError::InvalidEscrowDuration
        );

        let state = &mut ctx.accounts.state;
        state.authority = ctx.accounts.authority.key();
        state.platform_wallet = ctx.accounts.platform_wallet.key();
        state.escrow_duration = escrow_duration;
        state.nft_program = Pubkey::default(); // Set later via update_nft_program
        state.total_escrows = 0;
        state.total_volume = 0;
        state.is_paused = false;
        state.bump = ctx.bumps.state;
        state.product_type_fees = [0u64; PRODUCT_TYPE_COUNT]; // All default (use PLATFORM_FEE_BASIS_POINTS)

        emit!(ProgramInitialized {
            authority: state.authority,
            platform_wallet: state.platform_wallet,
            escrow_duration,
        });

        Ok(())
    }

    // =========================================================================
    // BUSINESS MANAGEMENT (Mirroring Solidity functions)
    // =========================================================================

    /// Register a new business
    /// 
    /// Similar to Solidity's registerBusiness()
    pub fn register_business(
        ctx: Context<RegisterBusiness>,
        business_id: String,
        business_name: String,
    ) -> Result<()> {
        require!(
            business_id.len() <= MAX_BUSINESS_ID_LEN,
            EscrowError::BusinessIdTooLong
        );
        require!(
            business_name.len() <= MAX_BUSINESS_NAME_LEN,
            EscrowError::BusinessNameTooLong
        );
        require!(!ctx.accounts.state.is_paused, EscrowError::ProgramPaused);

        let business = &mut ctx.accounts.business;
        let clock = Clock::get()?;

        business.business_id = business_id.clone();
        business.wallet_address = ctx.accounts.business_wallet.key();
        business.business_name = business_name.clone();
        business.status = BusinessStatus::Active;
        business.registration_date = clock.unix_timestamp;
        business.total_transactions = 0;
        business.total_volume = 0;
        business.nft_enabled = true; // Default: NFT enabled
        business.bump = ctx.bumps.business;

        emit!(BusinessRegistered {
            business_id: business_id.clone(),
            wallet_address: business.wallet_address,
            business_name,
        });

        Ok(())
    }

    /// Update business wallet address
    /// 
    /// Similar to Solidity's updateBusinessWallet()
    pub fn update_business_wallet(
        ctx: Context<UpdateBusinessWallet>,
        _business_id: String,
    ) -> Result<()> {
        let business = &mut ctx.accounts.business;
        let old_wallet = business.wallet_address;
        let new_wallet = ctx.accounts.new_wallet.key();

        require!(old_wallet != new_wallet, EscrowError::SameWalletAddress);

        business.wallet_address = new_wallet;

        emit!(BusinessWalletUpdated {
            business_id: business.business_id.clone(),
            old_wallet,
            new_wallet,
        });

        Ok(())
    }

    /// Suspend a business
    pub fn suspend_business(ctx: Context<AdminBusinessAction>, _business_id: String) -> Result<()> {
        let business = &mut ctx.accounts.business;
        require!(
            business.status != BusinessStatus::Suspended,
            EscrowError::BusinessAlreadySuspended
        );

        business.status = BusinessStatus::Suspended;

        emit!(BusinessSuspended {
            business_id: business.business_id.clone(),
            suspended_by: ctx.accounts.authority.key(),
        });

        Ok(())
    }

    /// Activate a suspended business
    pub fn activate_business(ctx: Context<AdminBusinessAction>, _business_id: String) -> Result<()> {
        let business = &mut ctx.accounts.business;
        require!(
            business.status == BusinessStatus::Suspended,
            EscrowError::BusinessNotSuspended
        );

        business.status = BusinessStatus::Active;

        emit!(BusinessActivated {
            business_id: business.business_id.clone(),
            activated_by: ctx.accounts.authority.key(),
        });

        Ok(())
    }

    /// Enable/disable NFT receipts for a business
    pub fn set_business_nft_enabled(
        ctx: Context<AdminBusinessAction>,
        _business_id: String,
        enabled: bool,
    ) -> Result<()> {
        let business = &mut ctx.accounts.business;
        business.nft_enabled = enabled;

        if enabled {
            emit!(BusinessNFTEnabled {
                business_id: business.business_id.clone(),
                updated_by: ctx.accounts.authority.key(),
            });
        } else {
            emit!(BusinessNFTDisabled {
                business_id: business.business_id.clone(),
                updated_by: ctx.accounts.authority.key(),
            });
        }

        Ok(())
    }

    // =========================================================================
    // ESCROW CREATION (Mirroring Solidity functions)
    // =========================================================================

    /// Create escrow with native SOL
    /// 
    /// Similar to Solidity's createEscrowETH()
    pub fn create_escrow_sol(
        ctx: Context<CreateEscrowSol>,
        escrow_id: String,
        business_id: String,
        amount: u64,
        metadata_uri: Option<String>,
        custom_release_at: Option<i64>,
    ) -> Result<()> {
        _create_escrow(
            &mut ctx.accounts.escrow,
            &mut ctx.accounts.business,
            &mut ctx.accounts.state,
            &ctx.accounts.customer,
            &ctx.accounts.system_program,
            escrow_id,
            business_id,
            NATIVE_SOL,
            amount,
            metadata_uri,
            custom_release_at,
            ctx.bumps.escrow,
        )?;

        // Transfer SOL to escrow PDA
        let cpi_context = CpiContext::new(
            ctx.accounts.system_program.to_account_info(),
            anchor_lang::system_program::Transfer {
                from: ctx.accounts.customer.to_account_info(),
                to: ctx.accounts.escrow.to_account_info(),
            },
        );
        anchor_lang::system_program::transfer(cpi_context, amount)?;

        Ok(())
    }

    /// Create escrow with SPL Token
    /// 
    /// Similar to Solidity's createEscrowERC20()
    pub fn create_escrow_spl(
        ctx: Context<CreateEscrowSpl>,
        escrow_id: String,
        business_id: String,
        amount: u64,
        metadata_uri: Option<String>,
        custom_release_at: Option<i64>,
    ) -> Result<()> {
        let token_mint = ctx.accounts.token_mint.key();

        _create_escrow(
            &mut ctx.accounts.escrow,
            &mut ctx.accounts.business,
            &mut ctx.accounts.state,
            &ctx.accounts.customer,
            &ctx.accounts.system_program,
            escrow_id,
            business_id,
            token_mint,
            amount,
            metadata_uri,
            custom_release_at,
            ctx.bumps.escrow,
        )?;

        // Transfer SPL tokens to escrow token account
        let cpi_accounts = SplTransfer {
            from: ctx.accounts.customer_token_account.to_account_info(),
            to: ctx.accounts.escrow_token_account.to_account_info(),
            authority: ctx.accounts.customer.to_account_info(),
        };
        let cpi_program = ctx.accounts.token_program.to_account_info();
        let cpi_ctx = CpiContext::new(cpi_program, cpi_accounts);
        token::transfer(cpi_ctx, amount)?;

        Ok(())
    }

    // =========================================================================
    // ESCROW MANAGEMENT (Mirroring Solidity functions)
    // =========================================================================

    /// Release funds after escrow period
    /// 
    /// Similar to Solidity's releaseFunds()
    pub fn release_funds_sol(ctx: Context<ReleaseFundsSol>, escrow_id: String) -> Result<()> {
        let escrow = &mut ctx.accounts.escrow;
        let business = &ctx.accounts.business;
        let clock = Clock::get()?;

        // Validate escrow state
        require!(
            escrow.status == TransactionStatus::Active,
            EscrowError::EscrowNotActive
        );
        require!(
            clock.unix_timestamp >= escrow.release_at,
            EscrowError::EscrowNotReady
        );

        // Authorization check: customer, business wallet, or authority
        let caller = ctx.accounts.caller.key();
        require!(
            caller == escrow.customer
                || caller == business.wallet_address
                || caller == ctx.accounts.state.authority,
            EscrowError::UnauthorizedRelease
        );

        // Mark as completed
        escrow.status = TransactionStatus::Completed;

        // Transfer business payout (SOL)
        let escrow_seeds = &[
            b"escrow",
            escrow_id.as_bytes(),
            &[escrow.bump],
        ];
        let signer_seeds = &[&escrow_seeds[..]];

        // Transfer to business
        **ctx.accounts.escrow.to_account_info().try_borrow_mut_lamports()? -= escrow.business_payout;
        **ctx.accounts.business_wallet.to_account_info().try_borrow_mut_lamports()? += escrow.business_payout;

        // Transfer platform fee
        **ctx.accounts.escrow.to_account_info().try_borrow_mut_lamports()? -= escrow.platform_fee;
        **ctx.accounts.platform_wallet.to_account_info().try_borrow_mut_lamports()? += escrow.platform_fee;

        emit!(FundsReleased {
            escrow_id: escrow.escrow_id.clone(),
            business_wallet: business.wallet_address,
            platform_wallet: ctx.accounts.platform_wallet.key(),
            business_amount: escrow.business_payout,
            platform_amount: escrow.platform_fee,
        });

        Ok(())
    }

    /// Release SPL token funds
    pub fn release_funds_spl(ctx: Context<ReleaseFundsSpl>, escrow_id: String) -> Result<()> {
        let escrow = &mut ctx.accounts.escrow;
        let business = &ctx.accounts.business;
        let clock = Clock::get()?;

        // Validate escrow state
        require!(
            escrow.status == TransactionStatus::Active,
            EscrowError::EscrowNotActive
        );
        require!(
            clock.unix_timestamp >= escrow.release_at,
            EscrowError::EscrowNotReady
        );

        // Authorization check
        let caller = ctx.accounts.caller.key();
        require!(
            caller == escrow.customer
                || caller == business.wallet_address
                || caller == ctx.accounts.state.authority,
            EscrowError::UnauthorizedRelease
        );

        // Mark as completed
        escrow.status = TransactionStatus::Completed;

        // Transfer SPL tokens using PDA signer
        let escrow_seeds = &[
            b"escrow",
            escrow_id.as_bytes(),
            &[escrow.bump],
        ];
        let signer_seeds = &[&escrow_seeds[..]];

        // Transfer to business
        let cpi_accounts_business = SplTransfer {
            from: ctx.accounts.escrow_token_account.to_account_info(),
            to: ctx.accounts.business_token_account.to_account_info(),
            authority: ctx.accounts.escrow.to_account_info(),
        };
        token::transfer(
            CpiContext::new_with_signer(
                ctx.accounts.token_program.to_account_info(),
                cpi_accounts_business,
                signer_seeds,
            ),
            escrow.business_payout,
        )?;

        // Transfer to platform
        let cpi_accounts_platform = SplTransfer {
            from: ctx.accounts.escrow_token_account.to_account_info(),
            to: ctx.accounts.platform_token_account.to_account_info(),
            authority: ctx.accounts.escrow.to_account_info(),
        };
        token::transfer(
            CpiContext::new_with_signer(
                ctx.accounts.token_program.to_account_info(),
                cpi_accounts_platform,
                signer_seeds,
            ),
            escrow.platform_fee,
        )?;

        emit!(FundsReleased {
            escrow_id: escrow.escrow_id.clone(),
            business_wallet: business.wallet_address,
            platform_wallet: ctx.accounts.state.platform_wallet,
            business_amount: escrow.business_payout,
            platform_amount: escrow.platform_fee,
        });

        Ok(())
    }

    /// Process refund (admin only)
    /// 
    /// Similar to Solidity's processRefund()
    pub fn process_refund_sol(ctx: Context<ProcessRefundSol>, escrow_id: String) -> Result<()> {
        let escrow = &mut ctx.accounts.escrow;

        require!(
            escrow.status == TransactionStatus::Active || escrow.status == TransactionStatus::Paused,
            EscrowError::CannotRefund
        );

        escrow.status = TransactionStatus::Refunded;
        escrow.refunded_by = Some(ctx.accounts.authority.key());

        // Refund full amount to customer
        let escrow_seeds = &[
            b"escrow",
            escrow_id.as_bytes(),
            &[escrow.bump],
        ];
        
        **ctx.accounts.escrow.to_account_info().try_borrow_mut_lamports()? -= escrow.amount;
        **ctx.accounts.customer.to_account_info().try_borrow_mut_lamports()? += escrow.amount;

        emit!(RefundProcessed {
            escrow_id: escrow.escrow_id.clone(),
            customer: escrow.customer,
            amount: escrow.amount,
            refunded_by: ctx.accounts.authority.key(),
        });

        Ok(())
    }

    /// Pause a transaction
    pub fn pause_transaction(ctx: Context<PauseTransaction>, _escrow_id: String) -> Result<()> {
        let escrow = &mut ctx.accounts.escrow;

        require!(
            escrow.status == TransactionStatus::Active,
            EscrowError::TransactionNotActive
        );

        escrow.status = TransactionStatus::Paused;

        emit!(TransactionPaused {
            escrow_id: escrow.escrow_id.clone(),
            paused_by: ctx.accounts.authority.key(),
        });

        Ok(())
    }

    /// Resume a paused transaction
    pub fn resume_transaction(ctx: Context<ResumeTransaction>, _escrow_id: String) -> Result<()> {
        let escrow = &mut ctx.accounts.escrow;

        require!(
            escrow.status == TransactionStatus::Paused,
            EscrowError::TransactionNotPaused
        );

        escrow.status = TransactionStatus::Active;

        emit!(TransactionResumed {
            escrow_id: escrow.escrow_id.clone(),
            resumed_by: ctx.accounts.authority.key(),
        });

        Ok(())
    }

    /// Update escrow release time (for event reschedules)
    /// 
    /// Similar to Solidity's updateEscrowReleaseTime()
    pub fn update_escrow_release_time(
        ctx: Context<UpdateReleaseTime>,
        _escrow_id: String,
        new_release_at: i64,
    ) -> Result<()> {
        let escrow = &mut ctx.accounts.escrow;
        let business = &ctx.accounts.business;
        let clock = Clock::get()?;

        require!(
            escrow.status == TransactionStatus::Active || escrow.status == TransactionStatus::Paused,
            EscrowError::TransactionNotActive
        );

        // Authorization: admin or business owner
        let caller = ctx.accounts.caller.key();
        require!(
            caller == ctx.accounts.state.authority || caller == business.wallet_address,
            EscrowError::UnauthorizedReleaseTimeUpdate
        );

        // Validate new release time
        require!(new_release_at > clock.unix_timestamp, EscrowError::ReleaseTimeInPast);
        require!(
            new_release_at <= clock.unix_timestamp + MAX_RELEASE_TIME,
            EscrowError::ReleaseTimeTooFar
        );
        require!(
            new_release_at >= escrow.release_at,
            EscrowError::CannotShortenReleaseTime
        );

        let old_release_at = escrow.release_at;
        escrow.release_at = new_release_at;

        emit!(EscrowReleaseTimeUpdated {
            escrow_id: escrow.escrow_id.clone(),
            old_release_at,
            new_release_at,
            updated_by: caller,
        });

        Ok(())
    }

    // =========================================================================
    // v3.0.0 — PRODUCT-TYPE-AWARE OPERATIONS
    // =========================================================================

    /// Create escrow with native SOL and product type (v3.0.0)
    pub fn create_escrow_sol_with_product_type(
        ctx: Context<CreateEscrowSol>,
        escrow_id: String,
        business_id: String,
        amount: u64,
        metadata_uri: Option<String>,
        custom_release_at: Option<i64>,
        product_type: u8,
    ) -> Result<()> {
        require!(
            (product_type as usize) < PRODUCT_TYPE_COUNT,
            EscrowError::InvalidProductType
        );

        _create_escrow_with_product_type(
            &mut ctx.accounts.escrow,
            &mut ctx.accounts.business,
            &mut ctx.accounts.state,
            &ctx.accounts.customer,
            &ctx.accounts.system_program,
            escrow_id,
            business_id,
            NATIVE_SOL,
            amount,
            metadata_uri,
            custom_release_at,
            ctx.bumps.escrow,
            product_type,
        )?;

        // Transfer SOL to escrow PDA
        let cpi_context = CpiContext::new(
            ctx.accounts.system_program.to_account_info(),
            anchor_lang::system_program::Transfer {
                from: ctx.accounts.customer.to_account_info(),
                to: ctx.accounts.escrow.to_account_info(),
            },
        );
        anchor_lang::system_program::transfer(cpi_context, amount)?;

        Ok(())
    }

    /// Create escrow with SPL token and product type (v3.0.0)
    pub fn create_escrow_spl_with_product_type(
        ctx: Context<CreateEscrowSpl>,
        escrow_id: String,
        business_id: String,
        amount: u64,
        metadata_uri: Option<String>,
        custom_release_at: Option<i64>,
        product_type: u8,
    ) -> Result<()> {
        require!(
            (product_type as usize) < PRODUCT_TYPE_COUNT,
            EscrowError::InvalidProductType
        );

        let token_mint = ctx.accounts.token_mint.key();

        _create_escrow_with_product_type(
            &mut ctx.accounts.escrow,
            &mut ctx.accounts.business,
            &mut ctx.accounts.state,
            &ctx.accounts.customer,
            &ctx.accounts.system_program,
            escrow_id,
            business_id,
            token_mint,
            amount,
            metadata_uri,
            custom_release_at,
            ctx.bumps.escrow,
            product_type,
        )?;

        // Transfer SPL tokens to escrow token account
        let cpi_accounts = SplTransfer {
            from: ctx.accounts.customer_token_account.to_account_info(),
            to: ctx.accounts.escrow_token_account.to_account_info(),
            authority: ctx.accounts.customer.to_account_info(),
        };
        let cpi_program = ctx.accounts.token_program.to_account_info();
        let cpi_ctx = CpiContext::new(cpi_program, cpi_accounts);
        token::transfer(cpi_ctx, amount)?;

        Ok(())
    }

    /// Set fee for a product type (admin only, v3.0.0)
    pub fn set_product_type_fee(
        ctx: Context<AdminAction>,
        product_type: u8,
        fee_bps: u64,
    ) -> Result<()> {
        require!(
            (product_type as usize) < PRODUCT_TYPE_COUNT,
            EscrowError::InvalidProductType
        );
        require!(fee_bps <= MAX_PRODUCT_TYPE_FEE, EscrowError::InvalidProductTypeFee);

        let state = &mut ctx.accounts.state;
        let old_fee = state.product_type_fees[product_type as usize];
        state.product_type_fees[product_type as usize] = fee_bps;

        emit!(ProductTypeFeeUpdated {
            product_type,
            old_fee,
            new_fee: fee_bps,
        });

        Ok(())
    }

    /// Admin override release time (can shorten AND extend, v3.0.0)
    ///
    /// Unlike update_escrow_release_time which only extends, this admin-only
    /// function can also shorten release times (e.g., for event cancellations).
    pub fn admin_override_release_time(
        ctx: Context<AdminOverrideReleaseTime>,
        _escrow_id: String,
        new_release_at: i64,
    ) -> Result<()> {
        let escrow = &mut ctx.accounts.escrow;
        let clock = Clock::get()?;

        require!(
            escrow.status == TransactionStatus::Active || escrow.status == TransactionStatus::Paused,
            EscrowError::TransactionNotActive
        );
        require!(new_release_at > clock.unix_timestamp, EscrowError::ReleaseTimeInPast);
        require!(
            new_release_at <= clock.unix_timestamp + MAX_RELEASE_TIME,
            EscrowError::ReleaseTimeTooFar
        );

        let old_release_at = escrow.release_at;
        escrow.release_at = new_release_at;

        emit!(AdminReleaseTimeOverride {
            escrow_id: escrow.escrow_id.clone(),
            old_release_at,
            new_release_at,
            admin: ctx.accounts.authority.key(),
        });

        Ok(())
    }

    // =========================================================================
    // ADMIN FUNCTIONS
    // =========================================================================

    /// Update escrow duration
    pub fn update_escrow_duration(ctx: Context<AdminAction>, new_duration: i64) -> Result<()> {
        require!(
            new_duration >= MIN_ESCROW_DURATION && new_duration <= MAX_ESCROW_DURATION,
            EscrowError::InvalidEscrowDuration
        );

        let state = &mut ctx.accounts.state;
        let old_duration = state.escrow_duration;
        state.escrow_duration = new_duration;

        emit!(EscrowDurationUpdated {
            old_duration,
            new_duration,
        });

        Ok(())
    }

    /// Update platform wallet
    pub fn update_platform_wallet(ctx: Context<UpdatePlatformWallet>) -> Result<()> {
        let state = &mut ctx.accounts.state;
        let old_wallet = state.platform_wallet;
        let new_wallet = ctx.accounts.new_platform_wallet.key();

        require!(old_wallet != new_wallet, EscrowError::SameWalletAddress);

        state.platform_wallet = new_wallet;

        emit!(PlatformWalletUpdated {
            old_wallet,
            new_wallet,
        });

        Ok(())
    }

    /// Update NFT program address
    pub fn update_nft_program(ctx: Context<AdminAction>, nft_program: Pubkey) -> Result<()> {
        let state = &mut ctx.accounts.state;
        let old_program = state.nft_program;
        state.nft_program = nft_program;

        emit!(NFTContractUpdated {
            old_contract: old_program,
            new_contract: nft_program,
        });

        Ok(())
    }

    /// Pause the program
    pub fn pause(ctx: Context<AdminAction>) -> Result<()> {
        ctx.accounts.state.is_paused = true;
        Ok(())
    }

    /// Unpause the program
    pub fn unpause(ctx: Context<AdminAction>) -> Result<()> {
        ctx.accounts.state.is_paused = false;
        Ok(())
    }

    /// Transfer authority
    pub fn transfer_authority(ctx: Context<TransferAuthority>) -> Result<()> {
        let state = &mut ctx.accounts.state;
        state.authority = ctx.accounts.new_authority.key();
        Ok(())
    }

    // =========================================================================
    // v3.3.0 — SUBSCRIPTION / MEMBERSHIP LINKING
    // =========================================================================

    /// Link an escrow to a subscription ID (for recurring billing audit trail)
    ///
    /// Similar to Solidity's linkEscrowToSubscription()
    pub fn link_escrow_to_subscription(
        ctx: Context<LinkSubscription>,
        _escrow_id: String,
        subscription_id: String,
    ) -> Result<()> {
        require!(!ctx.accounts.state.is_paused, EscrowError::ProgramPaused);

        let escrow = &mut ctx.accounts.escrow;
        escrow.subscription_id = Some(subscription_id.clone());

        emit!(SubscriptionEscrowLinked {
            escrow_id: escrow.escrow_id.clone(),
            subscription_id,
        });

        Ok(())
    }

    // =========================================================================
    // v3.4.0 — SPL TOKEN REFUND
    // =========================================================================

    /// Process refund for SPL token escrow (admin only)
    pub fn process_refund_spl(ctx: Context<ProcessRefundSpl>, escrow_id: String) -> Result<()> {
        let escrow = &mut ctx.accounts.escrow;

        require!(
            escrow.status == TransactionStatus::Active || escrow.status == TransactionStatus::Paused,
            EscrowError::CannotRefund
        );

        escrow.status = TransactionStatus::Refunded;
        escrow.refunded_by = Some(ctx.accounts.authority.key());

        // Refund full amount via SPL transfer to customer
        let escrow_seeds = &[
            b"escrow",
            escrow_id.as_bytes(),
            &[escrow.bump],
        ];
        let signer_seeds = &[&escrow_seeds[..]];

        let cpi_accounts = SplTransfer {
            from: ctx.accounts.escrow_token_account.to_account_info(),
            to: ctx.accounts.customer_token_account.to_account_info(),
            authority: ctx.accounts.escrow.to_account_info(),
        };
        token::transfer(
            CpiContext::new_with_signer(
                ctx.accounts.token_program.to_account_info(),
                cpi_accounts,
                signer_seeds,
            ),
            escrow.amount,
        )?;

        emit!(RefundProcessed {
            escrow_id: escrow.escrow_id.clone(),
            customer: escrow.customer,
            amount: escrow.amount,
            refunded_by: ctx.accounts.authority.key(),
        });

        Ok(())
    }

    // =========================================================================
    // v3.4.0 — TOKEN WHITELIST
    // =========================================================================

    /// Whitelist an SPL token for escrow usage
    ///
    /// Similar to Solidity's whitelistToken()
    pub fn whitelist_token(
        ctx: Context<WhitelistToken>,
        token_mint: Pubkey,
        min_amount: u64,
        decimals: u8,
        symbol: String,
    ) -> Result<()> {
        let config = &mut ctx.accounts.token_config;
        config.token_mint = token_mint;
        config.is_active = true;
        config.min_amount = min_amount;
        config.decimals = decimals;
        config.symbol = symbol.clone();
        config.bump = ctx.bumps.token_config;

        emit!(TokenWhitelisted {
            token_mint,
            min_amount,
            symbol,
        });

        Ok(())
    }

    /// Remove a token from the whitelist
    ///
    /// Similar to Solidity's removeToken()
    pub fn remove_token(ctx: Context<RemoveToken>, _token_mint: Pubkey) -> Result<()> {
        let config = &mut ctx.accounts.token_config;
        let token_mint = config.token_mint;
        config.is_active = false;

        emit!(TokenRemoved { token_mint });

        Ok(())
    }
}

// =============================================================================
// HELPER FUNCTIONS
// =============================================================================

fn _create_escrow<'info>(
    escrow: &mut Account<'info, Escrow>,
    business: &mut Account<'info, Business>,
    state: &mut Account<'info, ProgramState>,
    customer: &Signer<'info>,
    _system_program: &Program<'info, System>,
    escrow_id: String,
    business_id: String,
    token_address: Pubkey,
    amount: u64,
    metadata_uri: Option<String>,
    custom_release_at: Option<i64>,
    bump: u8,
) -> Result<()> {
    _create_escrow_with_product_type(
        escrow, business, state, customer, _system_program,
        escrow_id, business_id, token_address, amount, metadata_uri,
        custom_release_at, bump, 0, // ProductType::General = 0
    )
}

fn _create_escrow_with_product_type<'info>(
    escrow: &mut Account<'info, Escrow>,
    business: &mut Account<'info, Business>,
    state: &mut Account<'info, ProgramState>,
    customer: &Signer<'info>,
    _system_program: &Program<'info, System>,
    escrow_id: String,
    business_id: String,
    token_address: Pubkey,
    amount: u64,
    metadata_uri: Option<String>,
    custom_release_at: Option<i64>,
    bump: u8,
    product_type: u8,
) -> Result<()> {
    let clock = Clock::get()?;

    // Validations
    require!(!state.is_paused, EscrowError::ProgramPaused);
    require!(
        escrow_id.len() <= MAX_ESCROW_ID_LEN,
        EscrowError::EscrowIdTooLong
    );
    require!(
        business.status == BusinessStatus::Active,
        EscrowError::BusinessNotActive
    );
    require!(amount >= MIN_SOL_AMOUNT, EscrowError::InsufficientAmount);

    // Calculate fees — use product-type-specific fee if set, otherwise default
    let pt_idx = product_type as usize;
    let fee_bps = if pt_idx < PRODUCT_TYPE_COUNT && state.product_type_fees[pt_idx] > 0 {
        state.product_type_fees[pt_idx]
    } else {
        PLATFORM_FEE_BASIS_POINTS // default 5%
    };
    let platform_fee = (amount * fee_bps) / BASIS_POINTS_DENOMINATOR;
    let business_payout = amount - platform_fee;

    // Determine release time
    let release_at = if let Some(custom_time) = custom_release_at {
        require!(custom_time > clock.unix_timestamp, EscrowError::ReleaseTimeInPast);
        require!(
            custom_time <= clock.unix_timestamp + MAX_RELEASE_TIME,
            EscrowError::ReleaseTimeTooFar
        );
        custom_time
    } else {
        clock.unix_timestamp + state.escrow_duration
    };

    // Initialize escrow
    escrow.escrow_id = escrow_id.clone();
    escrow.customer = customer.key();
    escrow.business_id = business_id;
    escrow.token_address = token_address;
    escrow.amount = amount;
    escrow.business_payout = business_payout;
    escrow.platform_fee = platform_fee;
    escrow.status = TransactionStatus::Active;
    escrow.created_at = clock.unix_timestamp;
    escrow.release_at = release_at;
    escrow.refunded_by = None;
    escrow.nft_token_id = None;
    escrow.nft_metadata_uri = metadata_uri;
    escrow.bump = bump;
    escrow.product_type = product_type;
    escrow.subscription_id = None;

    // Update business stats
    business.total_transactions += 1;
    business.total_volume += amount;

    // Update program stats
    state.total_escrows += 1;
    state.total_volume += amount;

    emit!(EscrowCreated {
        escrow_id,
        customer: customer.key(),
        business_id: business.business_id.clone(),
        token_address,
        amount,
    });

    Ok(())
}

// =============================================================================
// ACCOUNT STRUCTS
// =============================================================================

#[account]
#[derive(InitSpace)]
pub struct ProgramState {
    /// Program authority (admin)
    pub authority: Pubkey,
    /// Platform wallet for fee collection
    pub platform_wallet: Pubkey,
    /// Default escrow duration in seconds
    pub escrow_duration: i64,
    /// NFT program for minting receipts
    pub nft_program: Pubkey,
    /// Total escrows created
    pub total_escrows: u64,
    /// Total volume processed
    pub total_volume: u64,
    /// Is program paused
    pub is_paused: bool,
    /// PDA bump
    pub bump: u8,
    /// v3.0.0: Product-type-specific fees (indexed by ProductType, 0 = use default)
    pub product_type_fees: [u64; 9],
}

#[account]
#[derive(InitSpace)]
pub struct Business {
    /// Unique business identifier
    #[max_len(64)]
    pub business_id: String,
    /// Business wallet address
    pub wallet_address: Pubkey,
    /// Business display name
    #[max_len(128)]
    pub business_name: String,
    /// Business status
    pub status: BusinessStatus,
    /// Registration timestamp
    pub registration_date: i64,
    /// Total transactions count
    pub total_transactions: u64,
    /// Total volume processed
    pub total_volume: u64,
    /// NFT receipts enabled
    pub nft_enabled: bool,
    /// PDA bump
    pub bump: u8,
}

#[account]
#[derive(InitSpace)]
pub struct Escrow {
    /// Unique escrow identifier
    #[max_len(64)]
    pub escrow_id: String,
    /// Customer pubkey
    pub customer: Pubkey,
    /// Business identifier
    #[max_len(64)]
    pub business_id: String,
    /// Token address (NATIVE_SOL for SOL, mint pubkey for SPL)
    pub token_address: Pubkey,
    /// Total amount escrowed
    pub amount: u64,
    /// Amount to be paid to business
    pub business_payout: u64,
    /// Platform fee amount
    pub platform_fee: u64,
    /// Transaction status
    pub status: TransactionStatus,
    /// Creation timestamp
    pub created_at: i64,
    /// Release timestamp
    pub release_at: i64,
    /// Who processed refund (if refunded)
    pub refunded_by: Option<Pubkey>,
    /// NFT token ID (if minted)
    pub nft_token_id: Option<u64>,
    /// NFT metadata URI
    #[max_len(256)]
    pub nft_metadata_uri: Option<String>,
    /// PDA bump
    pub bump: u8,
    /// v3.0.0: Product type for fee calculation
    pub product_type: u8,
    /// v3.3.0: Subscription ID for membership/pass recurring billing
    #[max_len(64)]
    pub subscription_id: Option<String>,
}

/// v3.4.0: Token whitelist configuration PDA
/// Mirrors Solidity's TokenConfig struct & whitelisted token mapping
#[account]
#[derive(InitSpace)]
pub struct TokenConfig {
    /// Token mint address
    pub token_mint: Pubkey,
    /// Whether the token is currently whitelisted
    pub is_active: bool,
    /// Minimum escrow amount for this token
    pub min_amount: u64,
    /// Token decimals
    pub decimals: u8,
    /// Token symbol
    #[max_len(16)]
    pub symbol: String,
    /// PDA bump
    pub bump: u8,
}

// =============================================================================
// ENUMS
// =============================================================================

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Copy, PartialEq, Eq, InitSpace)]
pub enum BusinessStatus {
    Pending,
    Active,
    Suspended,
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Copy, PartialEq, Eq, InitSpace)]
pub enum TransactionStatus {
    Pending,
    Active,
    Paused,
    Completed,
    Refunded,
    Failed,
}

/// Product type for variable fee calculation (v3.0.0)
/// Mirrors EVM ProductType enum
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Copy, PartialEq, Eq, InitSpace)]
#[repr(u8)]
pub enum ProductType {
    General = 0,
    Event = 1,
    Experience = 2,
    Stay = 3,
    Travel = 4,
    PpvStream = 5,
    Restaurant = 6,
    Membership = 7,
    Pass = 8,
}

// =============================================================================
// CONTEXT STRUCTS (Account Validation)
// =============================================================================

#[derive(Accounts)]
pub struct Initialize<'info> {
    #[account(
        init,
        payer = authority,
        space = 8 + ProgramState::INIT_SPACE,
        seeds = [b"state"],
        bump
    )]
    pub state: Account<'info, ProgramState>,
    
    /// CHECK: Platform wallet for fee collection
    pub platform_wallet: AccountInfo<'info>,
    
    #[account(mut)]
    pub authority: Signer<'info>,
    
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
#[instruction(business_id: String)]
pub struct RegisterBusiness<'info> {
    #[account(
        init,
        payer = authority,
        space = 8 + Business::INIT_SPACE,
        seeds = [b"business", business_id.as_bytes()],
        bump
    )]
    pub business: Account<'info, Business>,
    
    #[account(
        mut,
        seeds = [b"state"],
        bump = state.bump,
        has_one = authority
    )]
    pub state: Account<'info, ProgramState>,
    
    /// CHECK: Business wallet address
    pub business_wallet: AccountInfo<'info>,
    
    #[account(mut)]
    pub authority: Signer<'info>,
    
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
#[instruction(business_id: String)]
pub struct UpdateBusinessWallet<'info> {
    #[account(
        mut,
        seeds = [b"business", business_id.as_bytes()],
        bump = business.bump
    )]
    pub business: Account<'info, Business>,
    
    #[account(
        seeds = [b"state"],
        bump = state.bump,
        has_one = authority
    )]
    pub state: Account<'info, ProgramState>,
    
    /// CHECK: New business wallet address
    pub new_wallet: AccountInfo<'info>,
    
    pub authority: Signer<'info>,
}

#[derive(Accounts)]
#[instruction(business_id: String)]
pub struct AdminBusinessAction<'info> {
    #[account(
        mut,
        seeds = [b"business", business_id.as_bytes()],
        bump = business.bump
    )]
    pub business: Account<'info, Business>,
    
    #[account(
        seeds = [b"state"],
        bump = state.bump,
        has_one = authority
    )]
    pub state: Account<'info, ProgramState>,
    
    pub authority: Signer<'info>,
}

#[derive(Accounts)]
#[instruction(escrow_id: String, business_id: String)]
pub struct CreateEscrowSol<'info> {
    #[account(
        init,
        payer = customer,
        space = 8 + Escrow::INIT_SPACE,
        seeds = [b"escrow", escrow_id.as_bytes()],
        bump
    )]
    pub escrow: Account<'info, Escrow>,
    
    #[account(
        mut,
        seeds = [b"business", business_id.as_bytes()],
        bump = business.bump
    )]
    pub business: Account<'info, Business>,
    
    #[account(
        mut,
        seeds = [b"state"],
        bump = state.bump
    )]
    pub state: Account<'info, ProgramState>,
    
    #[account(mut)]
    pub customer: Signer<'info>,
    
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
#[instruction(escrow_id: String, business_id: String)]
pub struct CreateEscrowSpl<'info> {
    #[account(
        init,
        payer = customer,
        space = 8 + Escrow::INIT_SPACE,
        seeds = [b"escrow", escrow_id.as_bytes()],
        bump
    )]
    pub escrow: Account<'info, Escrow>,
    
    #[account(
        mut,
        seeds = [b"business", business_id.as_bytes()],
        bump = business.bump
    )]
    pub business: Account<'info, Business>,
    
    #[account(
        mut,
        seeds = [b"state"],
        bump = state.bump
    )]
    pub state: Account<'info, ProgramState>,
    
    pub token_mint: Account<'info, token::Mint>,
    
    #[account(mut)]
    pub customer_token_account: Account<'info, TokenAccount>,
    
    #[account(
        init_if_needed,
        payer = customer,
        associated_token::mint = token_mint,
        associated_token::authority = escrow
    )]
    pub escrow_token_account: Account<'info, TokenAccount>,
    
    #[account(mut)]
    pub customer: Signer<'info>,
    
    pub token_program: Program<'info, Token>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
#[instruction(escrow_id: String)]
pub struct ReleaseFundsSol<'info> {
    #[account(
        mut,
        seeds = [b"escrow", escrow_id.as_bytes()],
        bump = escrow.bump
    )]
    pub escrow: Account<'info, Escrow>,
    
    #[account(
        seeds = [b"business", escrow.business_id.as_bytes()],
        bump = business.bump
    )]
    pub business: Account<'info, Business>,
    
    #[account(
        seeds = [b"state"],
        bump = state.bump
    )]
    pub state: Account<'info, ProgramState>,
    
    /// CHECK: Business wallet to receive payout
    #[account(mut)]
    pub business_wallet: AccountInfo<'info>,
    
    /// CHECK: Platform wallet to receive fee
    #[account(mut)]
    pub platform_wallet: AccountInfo<'info>,
    
    pub caller: Signer<'info>,
    
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
#[instruction(escrow_id: String)]
pub struct ReleaseFundsSpl<'info> {
    #[account(
        mut,
        seeds = [b"escrow", escrow_id.as_bytes()],
        bump = escrow.bump
    )]
    pub escrow: Account<'info, Escrow>,
    
    #[account(
        seeds = [b"business", escrow.business_id.as_bytes()],
        bump = business.bump
    )]
    pub business: Account<'info, Business>,
    
    #[account(
        seeds = [b"state"],
        bump = state.bump
    )]
    pub state: Account<'info, ProgramState>,
    
    #[account(mut)]
    pub escrow_token_account: Account<'info, TokenAccount>,
    
    #[account(mut)]
    pub business_token_account: Account<'info, TokenAccount>,
    
    #[account(mut)]
    pub platform_token_account: Account<'info, TokenAccount>,
    
    pub caller: Signer<'info>,
    
    pub token_program: Program<'info, Token>,
}

#[derive(Accounts)]
#[instruction(escrow_id: String)]
pub struct ProcessRefundSol<'info> {
    #[account(
        mut,
        seeds = [b"escrow", escrow_id.as_bytes()],
        bump = escrow.bump
    )]
    pub escrow: Account<'info, Escrow>,
    
    #[account(
        seeds = [b"state"],
        bump = state.bump,
        has_one = authority
    )]
    pub state: Account<'info, ProgramState>,
    
    /// CHECK: Customer to receive refund
    #[account(mut)]
    pub customer: AccountInfo<'info>,
    
    pub authority: Signer<'info>,
    
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
#[instruction(escrow_id: String)]
pub struct PauseTransaction<'info> {
    #[account(
        mut,
        seeds = [b"escrow", escrow_id.as_bytes()],
        bump = escrow.bump
    )]
    pub escrow: Account<'info, Escrow>,
    
    #[account(
        seeds = [b"state"],
        bump = state.bump,
        has_one = authority
    )]
    pub state: Account<'info, ProgramState>,
    
    pub authority: Signer<'info>,
}

#[derive(Accounts)]
#[instruction(escrow_id: String)]
pub struct ResumeTransaction<'info> {
    #[account(
        mut,
        seeds = [b"escrow", escrow_id.as_bytes()],
        bump = escrow.bump
    )]
    pub escrow: Account<'info, Escrow>,
    
    #[account(
        seeds = [b"state"],
        bump = state.bump,
        has_one = authority
    )]
    pub state: Account<'info, ProgramState>,
    
    pub authority: Signer<'info>,
}

#[derive(Accounts)]
#[instruction(escrow_id: String)]
pub struct UpdateReleaseTime<'info> {
    #[account(
        mut,
        seeds = [b"escrow", escrow_id.as_bytes()],
        bump = escrow.bump
    )]
    pub escrow: Account<'info, Escrow>,
    
    #[account(
        seeds = [b"business", escrow.business_id.as_bytes()],
        bump = business.bump
    )]
    pub business: Account<'info, Business>,
    
    #[account(
        seeds = [b"state"],
        bump = state.bump
    )]
    pub state: Account<'info, ProgramState>,
    
    pub caller: Signer<'info>,
}

/// v3.0.0: Admin override release time (can shorten AND extend)
#[derive(Accounts)]
#[instruction(escrow_id: String)]
pub struct AdminOverrideReleaseTime<'info> {
    #[account(
        mut,
        seeds = [b"escrow", escrow_id.as_bytes()],
        bump = escrow.bump
    )]
    pub escrow: Account<'info, Escrow>,

    #[account(
        seeds = [b"state"],
        bump = state.bump,
        has_one = authority
    )]
    pub state: Account<'info, ProgramState>,

    pub authority: Signer<'info>,
}

#[derive(Accounts)]
pub struct AdminAction<'info> {
    #[account(
        mut,
        seeds = [b"state"],
        bump = state.bump,
        has_one = authority
    )]
    pub state: Account<'info, ProgramState>,
    
    pub authority: Signer<'info>,
}

#[derive(Accounts)]
pub struct UpdatePlatformWallet<'info> {
    #[account(
        mut,
        seeds = [b"state"],
        bump = state.bump,
        has_one = authority
    )]
    pub state: Account<'info, ProgramState>,
    
    /// CHECK: New platform wallet
    pub new_platform_wallet: AccountInfo<'info>,
    
    pub authority: Signer<'info>,
}

#[derive(Accounts)]
pub struct TransferAuthority<'info> {
    #[account(
        mut,
        seeds = [b"state"],
        bump = state.bump,
        has_one = authority
    )]
    pub state: Account<'info, ProgramState>,
    
    /// CHECK: New authority
    pub new_authority: AccountInfo<'info>,
    
    pub authority: Signer<'info>,
}

// v3.3.0 — Subscription linking context

#[derive(Accounts)]
#[instruction(escrow_id: String)]
pub struct LinkSubscription<'info> {
    #[account(
        mut,
        seeds = [b"escrow", escrow_id.as_bytes()],
        bump = escrow.bump,
    )]
    pub escrow: Account<'info, Escrow>,

    #[account(
        seeds = [b"state"],
        bump = state.bump,
        has_one = authority,
    )]
    pub state: Account<'info, ProgramState>,

    pub authority: Signer<'info>,
}

// v3.4.0 — SPL Token Refund context

#[derive(Accounts)]
#[instruction(escrow_id: String)]
pub struct ProcessRefundSpl<'info> {
    #[account(
        mut,
        seeds = [b"escrow", escrow_id.as_bytes()],
        bump = escrow.bump,
    )]
    pub escrow: Account<'info, Escrow>,

    #[account(
        seeds = [b"state"],
        bump = state.bump,
        has_one = authority,
    )]
    pub state: Account<'info, ProgramState>,

    #[account(
        mut,
        constraint = escrow_token_account.owner == escrow.key(),
    )]
    pub escrow_token_account: Account<'info, TokenAccount>,

    #[account(
        mut,
        constraint = customer_token_account.owner == escrow.customer,
    )]
    pub customer_token_account: Account<'info, TokenAccount>,

    pub authority: Signer<'info>,

    pub token_program: Program<'info, Token>,
}

// v3.4.0 — Token Whitelist contexts

#[derive(Accounts)]
#[instruction(token_mint: Pubkey)]
pub struct WhitelistToken<'info> {
    #[account(
        init_if_needed,
        payer = authority,
        space = 8 + TokenConfig::INIT_SPACE,
        seeds = [b"token_config", token_mint.as_ref()],
        bump,
    )]
    pub token_config: Account<'info, TokenConfig>,

    #[account(
        seeds = [b"state"],
        bump = state.bump,
        has_one = authority,
    )]
    pub state: Account<'info, ProgramState>,

    #[account(mut)]
    pub authority: Signer<'info>,

    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
#[instruction(token_mint: Pubkey)]
pub struct RemoveToken<'info> {
    #[account(
        mut,
        seeds = [b"token_config", token_mint.as_ref()],
        bump = token_config.bump,
    )]
    pub token_config: Account<'info, TokenConfig>,

    #[account(
        seeds = [b"state"],
        bump = state.bump,
        has_one = authority,
    )]
    pub state: Account<'info, ProgramState>,

    pub authority: Signer<'info>,
}

// =============================================================================
// EVENTS (Mirroring Solidity events)
// =============================================================================

#[event]
pub struct ProgramInitialized {
    pub authority: Pubkey,
    pub platform_wallet: Pubkey,
    pub escrow_duration: i64,
}

#[event]
pub struct BusinessRegistered {
    pub business_id: String,
    pub wallet_address: Pubkey,
    pub business_name: String,
}

#[event]
pub struct BusinessWalletUpdated {
    pub business_id: String,
    pub old_wallet: Pubkey,
    pub new_wallet: Pubkey,
}

#[event]
pub struct BusinessSuspended {
    pub business_id: String,
    pub suspended_by: Pubkey,
}

#[event]
pub struct BusinessActivated {
    pub business_id: String,
    pub activated_by: Pubkey,
}

#[event]
pub struct BusinessNFTEnabled {
    pub business_id: String,
    pub updated_by: Pubkey,
}

#[event]
pub struct BusinessNFTDisabled {
    pub business_id: String,
    pub updated_by: Pubkey,
}

#[event]
pub struct EscrowCreated {
    pub escrow_id: String,
    pub customer: Pubkey,
    pub business_id: String,
    pub token_address: Pubkey,
    pub amount: u64,
}

#[event]
pub struct FundsReleased {
    pub escrow_id: String,
    pub business_wallet: Pubkey,
    pub platform_wallet: Pubkey,
    pub business_amount: u64,
    pub platform_amount: u64,
}

#[event]
pub struct RefundProcessed {
    pub escrow_id: String,
    pub customer: Pubkey,
    pub amount: u64,
    pub refunded_by: Pubkey,
}

#[event]
pub struct TransactionPaused {
    pub escrow_id: String,
    pub paused_by: Pubkey,
}

#[event]
pub struct TransactionResumed {
    pub escrow_id: String,
    pub resumed_by: Pubkey,
}

#[event]
pub struct EscrowReleaseTimeUpdated {
    pub escrow_id: String,
    pub old_release_at: i64,
    pub new_release_at: i64,
    pub updated_by: Pubkey,
}

#[event]
pub struct EscrowDurationUpdated {
    pub old_duration: i64,
    pub new_duration: i64,
}

#[event]
pub struct PlatformWalletUpdated {
    pub old_wallet: Pubkey,
    pub new_wallet: Pubkey,
}

#[event]
pub struct NFTContractUpdated {
    pub old_contract: Pubkey,
    pub new_contract: Pubkey,
}

#[event]
pub struct NFTMintedForEscrow {
    pub escrow_id: String,
    pub token_id: u64,
    pub owner: Pubkey,
    pub metadata_uri: String,
}

// v3.0.0 Events

#[event]
pub struct ProductTypeFeeUpdated {
    pub product_type: u8,
    pub old_fee: u64,
    pub new_fee: u64,
}

#[event]
pub struct AdminReleaseTimeOverride {
    pub escrow_id: String,
    pub old_release_at: i64,
    pub new_release_at: i64,
    pub admin: Pubkey,
}

// v3.3.0 Events

#[event]
pub struct SubscriptionEscrowLinked {
    pub escrow_id: String,
    pub subscription_id: String,
}

// v3.4.0 Events

#[event]
pub struct TokenWhitelisted {
    pub token_mint: Pubkey,
    pub min_amount: u64,
    pub symbol: String,
}

#[event]
pub struct TokenRemoved {
    pub token_mint: Pubkey,
}

// =============================================================================
// ERRORS (Mirroring Solidity custom errors)
// =============================================================================

#[error_code]
pub enum EscrowError {
    #[msg("Business already exists")]
    BusinessAlreadyExists,
    
    #[msg("Business not found")]
    BusinessNotFound,
    
    #[msg("Business is not active")]
    BusinessNotActive,
    
    #[msg("Business already suspended")]
    BusinessAlreadySuspended,
    
    #[msg("Business is not suspended")]
    BusinessNotSuspended,
    
    #[msg("Escrow not found")]
    EscrowNotFound,
    
    #[msg("Token not supported")]
    TokenNotSupported,
    
    #[msg("Insufficient amount")]
    InsufficientAmount,
    
    #[msg("Escrow not ready for release")]
    EscrowNotReady,
    
    #[msg("Escrow is not active")]
    EscrowNotActive,
    
    #[msg("Cannot refund - invalid status")]
    CannotRefund,
    
    #[msg("Unauthorized refund")]
    UnauthorizedRefund,
    
    #[msg("Unauthorized release")]
    UnauthorizedRelease,
    
    #[msg("Transaction is not active")]
    TransactionNotActive,
    
    #[msg("Transaction is not paused")]
    TransactionNotPaused,
    
    #[msg("Invalid escrow duration")]
    InvalidEscrowDuration,
    
    #[msg("Duplicate escrow ID")]
    DuplicateEscrowId,
    
    #[msg("Wallet already assigned to another business")]
    WalletAlreadyAssigned,
    
    #[msg("Same wallet address")]
    SameWalletAddress,
    
    #[msg("Program is paused")]
    ProgramPaused,
    
    #[msg("Business ID too long")]
    BusinessIdTooLong,
    
    #[msg("Business name too long")]
    BusinessNameTooLong,
    
    #[msg("Escrow ID too long")]
    EscrowIdTooLong,
    
    #[msg("Release time is in the past")]
    ReleaseTimeInPast,
    
    #[msg("Release time is too far in the future")]
    ReleaseTimeTooFar,
    
    #[msg("Cannot shorten release time")]
    CannotShortenReleaseTime,
    
    #[msg("Unauthorized release time update")]
    UnauthorizedReleaseTimeUpdate,
    
    #[msg("Transfer failed")]
    TransferFailed,
    
    #[msg("Invalid address")]
    InvalidAddress,

    // v3.0.0 errors
    #[msg("Invalid product type")]
    InvalidProductType,

    #[msg("Invalid product type fee (max 50%)")]
    InvalidProductTypeFee,

    // v3.4.0 errors
    #[msg("Token is not whitelisted")]
    TokenNotWhitelisted,

    #[msg("Token is already whitelisted")]
    TokenAlreadyWhitelisted,
}
