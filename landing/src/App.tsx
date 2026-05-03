/**
 * @license
 * SPDX-License-Identifier: Apache-2.0
 */

import { motion } from 'motion/react';
import { Wallet, Info, Ban, Ticket, Coins } from 'lucide-react';

export default function App() {
  const containerVariants = {
    hidden: { opacity: 0 },
    visible: {
      opacity: 1,
      transition: {
        staggerChildren: 0.1
      }
    }
  };

  const itemVariants = {
    hidden: { opacity: 0, y: 20 },
    visible: { opacity: 1, y: 0 }
  };

  return (
    <div className="min-h-screen bg-surface selection:bg-white selection:text-black overflow-x-hidden">
      {/* TopAppBar */}
      <header className="fixed top-0 w-full z-50 border-b border-surface-container-high bg-surface/90 backdrop-blur-md">
        <div className="flex justify-between items-center px-6 py-4 max-w-7xl mx-auto w-full">
          <div className="flex items-center gap-2 group cursor-pointer active:scale-95 transition-all duration-150">
            <Coins className="w-6 h-6 text-primary" />
            <span className="text-ticket text-2xl tracking-tighter">DaddyX</span>
          </div>
          <button className="bg-white text-black px-6 py-2 rounded-full font-bold text-xs uppercase tracking-wider hover:scale-105 transition-all duration-200 glow-white-hover active:scale-95 ring-0 outline-none">
            Connect Wallet
          </button>
        </div>
      </header>

      <main className="pt-24">
        {/* HERO */}
        <section className="min-h-[80vh] flex flex-col justify-center items-center px-6 text-center relative">
          <div className="absolute inset-0 bg-[radial-gradient(circle_at_50%_40%,rgba(255,255,255,0.03)_0%,transparent_50%)] pointer-events-none" />
          
          <motion.div 
            initial="hidden"
            whileInView="visible"
            viewport={{ once: true }}
            variants={containerVariants}
            className="max-w-3xl z-10 space-y-8"
          >
            <motion.h1 variants={itemVariants} className="font-display text-5xl md:text-7xl font-bold uppercase tracking-tight text-white leading-[1.1]">
              BACK THE EVENT.<br />
              <span className="text-primary">EARN FROM THE NIGHT.</span>
            </motion.h1>

            <motion.p variants={itemVariants} className="text-lg md:text-xl text-neutral-400 max-w-xl mx-auto leading-relaxed font-light">
              DaddyX lets fans pre-finance events and earn a share of verified ticket revenue. Powered by Solana and built on battle-tested infrastructure.
            </motion.p>

            <motion.div variants={itemVariants} className="flex flex-col sm:flex-row gap-4 justify-center pt-4">
              <button className="bg-white text-black px-10 py-4 rounded-full font-bold text-sm uppercase tracking-wider hover:scale-105 transition-all duration-200 glow-white-hover flex items-center justify-center gap-2">
                <Wallet className="w-4 h-4" />
                Connect Wallet
              </button>
              <button className="border border-white/20 text-white px-10 py-4 rounded-full font-bold text-sm uppercase tracking-wider hover:bg-white/5 transition-all duration-200 hover:scale-105 flex items-center justify-center gap-2">
                <Info className="w-4 h-4" />
                How It Works
              </button>
            </motion.div>

            <motion.div variants={itemVariants} className="flex flex-wrap justify-center gap-3 pt-12">
              <span className="bg-white/5 text-white/80 px-4 py-2 rounded-full text-xs font-bold uppercase tracking-widest border border-white/10 flex items-center gap-2">
                <div className="w-1.5 h-1.5 rounded-full bg-primary animate-pulse" />
                Powered by Solana
              </span>
              <span className="bg-white/5 text-white/60 px-4 py-2 rounded-full text-xs font-bold uppercase tracking-widest border border-white/5">
                Built on TicketDaddy
              </span>
              <span className="bg-white/5 text-white/60 px-4 py-2 rounded-full text-xs font-bold uppercase tracking-widest border border-white/5">
                600K+ Tickets Processed
              </span>
            </motion.div>
          </motion.div>
        </section>

        {/* THE PROBLEM */}
        <section className="py-24 px-6 max-w-6xl mx-auto">
          <motion.h2 
            initial={{ opacity: 0 }}
            whileInView={{ opacity: 1 }}
            viewport={{ once: true }}
            className="font-display text-4xl md:text-5xl text-center text-white mb-16 uppercase tracking-tighter italic"
          >
            THE BROKEN SYSTEM
          </motion.h2>

          <div className="grid md:grid-cols-2 gap-8">
            {/* Card 1 */}
            <motion.div 
              initial={{ opacity: 0, x: -20 }}
              whileInView={{ opacity: 1, x: 0 }}
              viewport={{ once: true }}
              className="bg-surface-container border border-outline-variant/30 p-8 flex flex-col gap-6 rounded-2xl relative overflow-hidden group hover:border-primary/30 transition-colors duration-300"
            >
              <div className="flex justify-between items-start">
                <div className="space-y-1">
                  <h3 className="text-2xl font-bold text-white leading-tight">The Organizer Burden</h3>
                  <p className="text-neutral-500 font-medium">Financing friction</p>
                </div>
                <div className="bg-primary/10 p-3 rounded-xl">
                  <Ban className="w-8 h-8 text-primary" />
                </div>
              </div>
              <p className="text-neutral-400 text-lg leading-relaxed">
                Organizers absorb 100% of upfront costs while waiting weeks for ticketing payouts to clear, stifling growth and event scale.
              </p>
              <div className="mt-8 pt-8 border-t border-outline-variant/20">
                <div className="flex items-baseline gap-2">
                  <span className="font-display text-5xl text-white">7-30</span>
                  <span className="text-neutral-500 font-bold uppercase tracking-widest text-sm">Days</span>
                </div>
                <span className="text-xs text-neutral-600 font-bold uppercase tracking-widest mt-1 block">Average Payout Delay</span>
              </div>
            </motion.div>

            {/* Card 2 */}
            <motion.div 
              initial={{ opacity: 0, x: 20 }}
              whileInView={{ opacity: 1, x: 0 }}
              viewport={{ once: true }}
              className="bg-surface-container border border-outline-variant/30 p-8 flex flex-col gap-6 rounded-2xl relative overflow-hidden group hover:border-secondary/30 transition-colors duration-300"
            >
              <div className="flex justify-between items-start">
                <div className="space-y-1">
                  <h3 className="text-2xl font-bold text-white leading-tight">The Fan Reality</h3>
                  <p className="text-neutral-500 font-medium">Zero ownership</p>
                </div>
                <div className="bg-secondary/10 p-3 rounded-xl">
                  <Ticket className="w-8 h-8 text-secondary" />
                </div>
              </div>
              <p className="text-neutral-400 text-lg leading-relaxed">
                Fans pour billions into the industry but capture zero upside from the financial success of the shows they hype and attend.
              </p>
              <div className="mt-8 pt-8 border-t border-outline-variant/20">
                <div className="flex items-baseline gap-2">
                  <span className="font-display text-5xl text-white">0%</span>
                </div>
                <span className="text-xs text-neutral-600 font-bold uppercase tracking-widest mt-1 block">Financial Upside for Fans</span>
              </div>
            </motion.div>
          </div>
        </section>

        {/* Footer */}
        <footer className="w-full border-t border-surface-container-high py-20 bg-surface mt-12">
          <div className="flex flex-col md:flex-row justify-between items-center px-6 max-w-7xl mx-auto gap-12">
            <div className="flex flex-col items-center md:items-start gap-4">
              <div className="text-ticket text-3xl tracking-tighter text-white">
                DaddyX
              </div>
              <p className="text-xs text-neutral-600 font-bold uppercase tracking-[0.2em]">
                Powering the Solana Event Economy
              </p>
            </div>

            <div className="flex flex-wrap gap-x-8 gap-y-4 justify-center">
              {['How It Works', 'For Organizers', 'For Fans', 'Privacy', 'Terms'].map((link) => (
                <a 
                  key={link}
                  className="text-xs font-bold uppercase tracking-widest text-neutral-500 hover:text-white transition-colors cursor-pointer"
                  href="#"
                >
                  {link}
                </a>
              ))}
            </div>

            <div className="text-[10px] font-bold uppercase tracking-[0.2em] text-neutral-700">
              &copy; 2026 DaddyX by TicketDaddy Inc. All rights reserved.
            </div>
          </div>
        </footer>
      </main>
    </div>
  );
}
