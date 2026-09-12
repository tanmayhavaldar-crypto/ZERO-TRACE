import React, { useState, useEffect, useRef } from "react";
import ERASINGTOOL from "./ERASINGTOOL.tsx"; 
import RECOVERTOOL from "./RECOVERTOOL.tsx";
import FORENSIC from "./FORENSIC.tsx";
import ABOUTUS from "./ABOUTUS.tsx";
import CONTACTUS from "./CONTACTUS.tsx";
import { invoke } from "@tauri-apps/api/core";


export default function App() {
  const [step, setStep] = useState(0);
  const [currentView, setCurrentView] = useState("home");
  const howToUseRef = useRef<HTMLDivElement>(null);
  const scrollToHowToUse = () => {
    howToUseRef.current?.scrollIntoView({ behavior: "smooth" });
  };
const [terminalOutput, setTerminalOutput] = useState([
    { text: "> NullTrace architecture ready.", color: "var(--text-main)" },
    { text: "> Awaiting target parameters...", color: "var(--text-main)" }
  ]);
   const terminalEndRef = useRef<HTMLDivElement>(null);
  const words = ["RECONSTRUCTION", "EXTRACTION", "EXCAVATION", "RECOVERY"];
  const [wordIndex, setWordIndex] = useState(0);
  const [animationStyle, setAnimationStyle] = useState({
    opacity: 1,
    transform: "translateY(0px)",
    transition: "opacity 0.4s ease, transform 0.4s ease",
  });
  //FIRST PAGE TEXT TRANSITION
  useEffect(() => {
    terminalEndRef.current?.scrollIntoView({ behavior: "smooth" });
  }, [terminalOutput]);
//  backend connection test
  const testBackendConnection = async () => {
    try {
      // This fires a secure message across the Tauri IPC bridge directly to Rust
      const response = await invoke("test_rust_bridge");
      alert("Backend says: " + response); 
    } catch (error) {
      alert("Bridge failed! Error: " + error);
    }
  };
  //STARTING OF PAGE ANIMATION
  useEffect(() => {
    if (currentView !== "home") return; // Only run animation on home screen

    const intervalId = setInterval(() => {
      setAnimationStyle({
        opacity: 0,
        transform: "translateY(-15px)",
        transition: "opacity 0.4s ease, transform 0.4s ease",
      });

      setTimeout(() => {
        setWordIndex((prev) => (prev + 1) % words.length);
        
        setAnimationStyle({
          opacity: 0,
          transform: "translateY(15px)",
          transition: "none",
        });

        setTimeout(() => {
          setAnimationStyle({
            opacity: 1,
            transform: "translateY(0px)",
            transition: "opacity 0.4s ease, transform 0.4s ease",
          });
        }, 50);
      }, 400);
    }, 2500);

    return () => clearInterval(intervalId);
  }, [currentView, words.length]);
  
  useEffect(() => {
    const timings = [
      { step: 1, time: 300 },
      { step: 2, time: 600 },
      { step: 3, time: 900 },
      { step: 4, time: 1200 },
      { step: 5, time: 1800 },
      { step: 6, time: 3000 },
      { step: 7, time: 4000 },
    ];

    const timeouts = timings.map((t) =>
      setTimeout(() => setStep(t.step), t.time)
    );

    // Z.E.R.O T.R.A.C.E. Backend Connection Test
  

    return () => timeouts.forEach(clearTimeout);
  }, []);

  const showSplash = step < 7;
  const fadeOutSplash = step >= 6;

  return (
    <>
      {
      //SPLASH SCREEN / INTRO LAYER (Z-50)
      }
      {showSplash && (
        <div
          className={`fixed inset-0 z-50 flex flex-col items-center justify-center bg-[#0a0a0c] transition-opacity duration-1000 ease-in-out ${
            fadeOutSplash ? "opacity-0" : "opacity-100"
          }`}
        >
          <div className="text-4xl md:text-7xl font-black tracking-[0.2em] flex items-center text-white drop-shadow-[0_0_15px_rgba(255,255,255,0.5)]">
            <span className={`transition-opacity duration-300 ${step >= 1 ? "opacity-100" : "opacity-0"}`}>Z</span>
            <span className={`transition-opacity duration-300 ${step >= 2 ? "opacity-100" : "opacity-0"}`}>E</span>
            <span className={`transition-opacity duration-300 ${step >= 3 ? "opacity-100" : "opacity-0"}`}>R</span>
            <span className={`transition-opacity duration-300 ${step >= 4 ? "opacity-100" : "opacity-0"}`}>O</span>
            
            <span 
              className={`flex items-center transition-all duration-700 ease-out overflow-hidden ${
                step >= 5 ? "opacity-100 max-w-[300px] ml-4" : "opacity-0 max-w-0 ml-0"
              }`}
            >
              TRACE
              <span className="text-red-500 drop-shadow-[0_0_15px_rgba(239,68,68,0.8)]">.</span>
            </span>
          </div>
          
          <div className={`mt-8 h-1 w-48 bg-gray-800 rounded-full overflow-hidden transition-opacity duration-500 ${step >= 1 ? "opacity-100" : "opacity-0"}`}>
            <div className="h-full bg-red-500 animate-[pulse_1.5s_ease-in-out_infinite] w-full"></div>
          </div>
        </div>
      )}

      {
        // HOME VIEW (Only renders if currentView === 'home')
          }
      {currentView === "home" && (
        <div className="min-h-screen bg-[#0a0a0c] text-white font-sans relative overflow-x-hidden selection:bg-red-500/30 flex flex-col">
          
          <div className="absolute inset-0 z-0 pointer-events-none">
            <img 
              src="https://images.unsplash.com/photo-1526374965328-7f61d4dc18c5?q=80&w=2070&auto=format&fit=crop" 
              alt="Cybersecurity Background" 
              className="w-full h-full object-cover opacity-30"
            />
            <div className="absolute inset-0 bg-gradient-to-r from-[#0a0a0c] via-[#0a0a0c]/80 to-transparent"></div>
            <div className="absolute inset-0 bg-gradient-to-t from-[#0a0a0c] via-transparent to-[#0a0a0c]/80"></div>
            <div className="absolute top-[-10%] left-[-10%] w-[50%] h-[50%] bg-red-900/30 blur-[120px] rounded-full mix-blend-screen"></div>
            <div className="absolute top-[20%] right-[-10%] w-[50%] h-[50%] bg-purple-900/20 blur-[120px] rounded-full mix-blend-screen"></div>
          </div>

          <div className="relative z-10 flex flex-col flex-1">
            <nav className="flex items-center justify-between px-8 py-6 max-w-7xl w-full mx-auto">
              <div 
                className="text-2xl font-black tracking-tighter cursor-pointer" 
                onClick={() => setCurrentView("home")}
              >
                ZERO TRACE<span className="text-red-500">.</span>
              </div>
              {/* ALL NAV BAR BUTTONS */}
              <div className="hidden md:flex items-center space-x-8 text-sm text-gray-300 font-medium">
                <button onClick={() => setCurrentView("erasing")} className="hover:text-white transition cursor-pointer">Erasing Tool</button>
                <button onClick={() => setCurrentView("recover")} className="hover:text-white transition cursor-pointer">Recover Tool</button>
                <button onClick={() => setCurrentView("forensic")} className="hover:text-white transition cursor-pointer">Forensic History</button>
                <button onClick={() => setCurrentView("aboutus")} className="hover:text-white transition cursor-pointer">About Us</button>
              </div>

              <div className="flex items-center space-x-4">
                <button onClick={() => setCurrentView("contactus")}className="hidden sm:block px-5 py-2 text-sm font-medium border border-gray-600 rounded hover:border-gray-400 transition bg-black/20 backdrop-blur-sm cursor-pointer">
                  Contact Us
                </button>
                <button className="px-5 py-2 text-sm font-medium bg-white text-black rounded hover:bg-gray-200 transition shadow-[0_0_15px_rgba(255,255,255,0.2)] cursor-pointer">
                  Protect Now
                </button>
              </div>
            </nav>
              {/* MAIN CONTENT */}
            <main className="flex-1 flex flex-col justify-center max-w-7xl mx-auto px-8 py-12 w-full">
                <div className="pt-[50px] pb-[110px] font-['Space_Mono'] font-bold">
              <h1 className="text-[clamp(3rem,7vw,6.5rem)] leading-[1.1] m-0 uppercase text-white">
                THE NEW<br />
                STANDARD IN<br />
                DATA{' '}
                <span className="text-red-500 text-[var(--coral)] inline-block" style={animationStyle}>
                  {words[wordIndex]}
                </span>
              </h1>
              {/* SCROLL DOWN LAYER OF HOMEPAGE */}
                <div className="flex flex-wrap gap-4 py-7">
                  <button 
                    onClick={scrollToHowToUse}
                    className="px-8 py-3.5 bg-white text-black font-semibold rounded transition cursor-pointer shadow-[0_0_20px_rgba(255,255,255,0.15)] delay-150 duration-300 ease-in-out hover:-translate-y-1 hover:scale-110 hover:bg-orange-500"
                  >
                    How to Use
                  </button>

                  <button onClick={testBackendConnection}>Test Backend Bridge</button>
                </div>
              </div>
              <section ref={howToUseRef} className="max-w-7xl mx-auto px-8 py-24 w-full border-t border-gray-800/50">
            <div className="mb-16">
              <h2 className="text-4xl md:text-5xl font-semibold leading-tight tracking-tight mb-4 drop-shadow-lg">
                Master Your Digital Environment
              </h2>
              <p className="text-gray-400 font-mono text-sm md:text-base border-l-2 border-red-500 pl-4">
                Execute secure forensics in three critical phases.
              </p>
            </div>

            <div className="grid grid-cols-1 md:grid-cols-3 gap-8">
              {/* Step 01 */}
              <div className="bg-[#111115]/80 backdrop-blur-xl border border-white/10 rounded-2xl p-8 shadow-xl hover:border-red-500/50 transition-colors group">
                <div className="text-red-500 font-mono font-bold text-xl mb-4 border-b border-gray-800 pb-2 inline-block">Step 01</div>
                <h3 className="text-xl font-medium text-white mb-2">Select Your Module</h3>
                <p className="text-gray-300 text-sm mb-4 font-medium">Choose your operational objective.</p>
                <p className="text-gray-500 text-sm font-mono leading-relaxed group-hover:text-gray-400 transition-colors">
                  Navigate the dashboard to select between Zero-Residue Erasure (for cryptographic storage wiping) or our Artifact Carving Engine (to reconstruct heavily fragmented or deleted data).
                </p>
              </div>

              {/* Step 02 */}
              <div className="bg-[#111115]/80 backdrop-blur-xl border border-white/10 rounded-2xl p-8 shadow-xl hover:border-purple-500/50 transition-colors group">
                <div className="text-purple-500 font-mono font-bold text-xl mb-4 border-b border-gray-800 pb-2 inline-block">Step 02</div>
                <h3 className="text-xl font-medium text-white mb-2">Secure the Sandbox</h3>
                <p className="text-gray-300 text-sm mb-4 font-medium">Isolate the target.</p>
                <p className="text-gray-500 text-sm font-mono leading-relaxed group-hover:text-gray-400 transition-colors">
                  For maximum safety, mount your target drive within a fully air-gapped Virtual Machine. Our architecture is designed to safely interact with raw storage sectors without risking your host operating system.
                </p>
              </div>

              {/* Step 03 */}
              <div className="bg-[#111115]/80 backdrop-blur-xl border border-white/10 rounded-2xl p-8 shadow-xl hover:border-blue-500/50 transition-colors group">
                <div className="text-blue-500 font-mono font-bold text-xl mb-4 border-b border-gray-800 pb-2 inline-block">Step 03</div>
                <h3 className="text-xl font-medium text-white mb-2">Execute Protocol</h3>
                <p className="text-gray-300 text-sm mb-4 font-medium">Deploy and monitor.</p>
                <p className="text-gray-500 text-sm font-mono leading-relaxed group-hover:text-gray-400 transition-colors">
                  Bypass the safety lock to authorize the low-level system scripts. Our FastAPI bridge will execute the operations in real-time, providing live progress updates as sectors are overwritten or carved.
                </p>
              </div>
            </div>
          </section>
            </main>
          </div>
        </div>
      )}

      {
          // TOOL VIEWS (Only one renders at a time)
          }
      {currentView === "erasing" && <ERASINGTOOL setCurrentView={setCurrentView} />}
      {currentView === "recover" && <RECOVERTOOL setCurrentView={setCurrentView} />}
      {currentView === "forensic" && <FORENSIC setCurrentView={setCurrentView} />}
      {currentView === "aboutus" && <ABOUTUS setCurrentView={setCurrentView} />}
      {currentView === "contactus" && <CONTACTUS setCurrentView={setCurrentView} />}    
    </>
  );
}