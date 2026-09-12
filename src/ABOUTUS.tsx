import React from "react";

interface AboutUsProps {
  setCurrentView: (view: string) => void;
}

export default function ABOUTUS({ setCurrentView }: AboutUsProps) {
  
  const teamMembers = [
    { 
      name: "TANMAY", 
      role: "UI/UX & Integration Lead", 
      desc: "Orchestrates the overall software architecture by linking backend processing modules with the frontend, ensuring seamless communication, data flow, and stability across the entire application.",
      color: "text-red-400"
    },
     { 
      name: "KRISHNA", 
      role: "Frontend & Deployment Lead", 
      desc: "Spearheads the development of the React/Tauri desktop client. Manages deployment pipelines, release schedules, and ensures a secure, responsive, and intuitive user interface.",
      color: "text-blue-400"
    },
    { 
      name: "RITHIK", 
      role: "Hardware Wiping Specialist", 
      desc: "Develops destructive modules and raw-storage overwrite algorithms. Ensures the application meets strict security standards for secure drive erasure and permanent hardware-level data destruction.",
      color: "text-red-400"
    },
    { 
      name: "VIDHATRI", 
      role: "File Sanitization Specialist", 
      desc: "Specializes in targeted data clearing. Develops algorithms for secure file deletion, metadata stripping, and precise data sanitization without compromising the surrounding file system or OS structures.",
      color: "text-blue-400"
    },
    { 
      name: "INCHARA", 
      role: "File Carving Specialist", 
      desc: "Focuses on backend data extraction logic. Builds the robust algorithms necessary to scan raw storage, identify specific file signatures, and accurately retrieve hidden or deleted digital artifacts..",
      color: "text-red-400"
    },
    { 
      name: "AHAN", 
      role: "Drive Reconstruction Specialist", 
      desc: "Handles complex storage recovery operations. Responsible for implementing algorithms that repair corrupted file systems, reassemble fragmented data structures, and logically reconstruct damaged or wiped drives.",
      color: "text-blue-400"
    }
  ];

  return (
    <div className="min-h-screen bg-[#0a0a0c] text-white font-sans relative overflow-x-hidden selection:bg-gray-500/30 flex flex-col">
      
      {/* BACKGROUND LAYER */}
      <div className="absolute inset-0 z-0 pointer-events-none fixed">
        <img 
          src="https://images.unsplash.com/photo-1526374965328-7f61d4dc18c5?q=80&w=2070&auto=format&fit=crop" 
          alt="Cybersecurity" 
          className="w-full h-full object-cover opacity-10"
        />
        <div className="absolute inset-0 bg-gradient-to-r from-[#0a0a0c] via-[#0a0a0c]/90 to-[#0a0a0c]/80"></div>
        <div className="absolute inset-0 bg-gradient-to-b from-[#0a0a0c] via-transparent to-[#0a0a0c]/90"></div>
        {/* Ambient Glows */}
        <div className="absolute top-[-10%] left-[10%] w-[30%] h-[40%] bg-red-900/10 blur-[120px] rounded-full mix-blend-screen"></div>
        <div className="absolute top-[30%] right-[-10%] w-[40%] h-[50%] bg-blue-900/10 blur-[120px] rounded-full mix-blend-screen"></div>
      </div>

      {/* FOREGROUND CONTENT LAYER */}
      <div className="relative z-10 flex flex-col flex-1 pb-16">
        
        {/* Navigation */}
        <header className="border-b border-white/5 bg-[#0a0a0c]/80 backdrop-blur-md sticky top-0 z-40">
          <nav className="flex items-center justify-between px-8 py-5 max-w-7xl w-full mx-auto">
            <div 
              className="text-2xl font-black tracking-tighter cursor-pointer hover:text-gray-300 transition"
              onClick={() => setCurrentView("home")}
            >
              ZERO TRACE<span className="text-red-500">.</span>
            </div>

            <div className="hidden md:flex items-center space-x-8 text-sm font-medium">
              <button onClick={() => setCurrentView("erasing")} className="text-gray-400 hover:text-white transition cursor-pointer">Erasing Tool</button>
              <button onClick={() => setCurrentView("recover")} className="text-gray-400 hover:text-white transition cursor-pointer">Recover Tool</button>
              <button onClick={() => setCurrentView("forensic")} className="text-gray-400 hover:text-white transition curson cursor-pointer">Forensic History</button>
              <button className="text-white relative cursor-pointer">
                About Us
                <span className="absolute -bottom-6 left-0 w-full h-[2px] bg-white"></span>
              </button>
            </div>

            <div className="flex items-center space-x-4">
              <button className="px-5 py-2 text-sm font-mono border border-gray-600 text-gray-300 rounded hover:bg-gray-500/10 transition">
                SIH 2026 // NTRO
              </button>
            </div>
          </nav>
        </header>

        {/* Main Content Area */}
        <main className="flex-1 flex flex-col max-w-7xl mx-auto px-4 pt-16 w-full">
          
          {/* Header */}
          <div className="text-center max-w-4xl mx-auto mb-16">
            <h1 className="text-4xl md:text-5xl font-semibold leading-tight tracking-tight mb-6 drop-shadow-lg">
              Pioneering Next-Generation <br className="hidden md:block"/> Digital Forensics.
            </h1>
            <p className="text-gray-400 text-base md:text-lg leading-relaxed font-mono">
              We are the engineering team behind Z.E.R.O T.R.A.C.E., an advanced suite designed to bridge the gap between secure data sanitation and critical artifact recovery. Built to meet the rigorous demands of modern cybersecurity, our platform provides forensic investigators and security professionals with uncompromising control over raw storage environments.
            </p>
          </div>

          {/* Core Engine Section */}
          <div className="w-full bg-[#111115]/80 backdrop-blur-xl border border-white/10 rounded-2xl shadow-[0_0_40px_rgba(0,0,0,0.8)] overflow-hidden mb-12">
            
            {/* The Name Banner */}
            <div className="bg-gradient-to-r from-gray-900 via-[#1a1a24] to-gray-900 border-b border-gray-800 p-8 text-center relative overflow-hidden">
              <div className="absolute inset-0 bg-[url('https://www.transparenttextures.com/patterns/carbon-fibre.png')] opacity-10 mix-blend-overlay"></div>
              <h2 className="text-xs font-mono text-gray-500 uppercase tracking-[0.3em] mb-4 relative z-10">The Engine: What is it?</h2>
              <p className="text-2xl md:text-3xl font-black tracking-widest text-white relative z-10">
                <span className="text-red-500">Z</span>.ero-residue <span className="text-red-500">E</span>.rasure & <span className="text-red-500">R</span>.aw-storage <span className="text-red-500">O</span>.verwrite <br className="hidden md:block mt-2"/>
                <span className="text-sm font-normal text-gray-400 mx-4 font-mono lowercase italic">with</span> <br className="hidden md:block mb-2"/>
                <span className="text-blue-500">T</span>.argeted <span className="text-blue-500">R</span>.econstruction & <span className="text-blue-500">A</span>.rtifact <span className="text-blue-500">C</span>.arving <span className="text-blue-500">E</span>.ngine
              </p>
            </div>

            {/* The Pillars */}
            <div className="p-8 md:p-12">
              <p className="text-gray-300 text-center max-w-3xl mx-auto mb-12 text-sm leading-relaxed">
                Rather than relying on disjointed scripts, we have unified two opposing pillars of digital forensics into a single, sandboxed environment.
              </p>

              <div className="grid grid-cols-1 md:grid-cols-2 gap-8">
                {/* Destructive Card */}
                <div className="bg-[#0a0a0c]/80 border border-red-500/20 rounded-xl p-6 relative group hover:border-red-500/50 transition-colors">
                  <div className="absolute top-0 left-0 w-full h-1 bg-gradient-to-r from-red-600 to-transparent opacity-50"></div>
                  <div className="flex items-center mb-4">
                    <div className="w-10 h-10 rounded bg-red-500/10 flex items-center justify-center mr-4 text-red-500">
                      <svg xmlns="http://www.w3.org/2000/svg" className="h-6 w-6" fill="none" viewBox="0 0 24 24" stroke="currentColor"><path strokeLinecap="round" strokeLinejoin="round" strokeWidth={1.5} d="M19 7l-.867 12.142A2 2 0 0116.138 21H7.862a2 2 0 01-1.995-1.858L5 7m5 4v6m4-6v6m1-10V4a1 1 0 00-1-1h-4a1 1 0 00-1 1v3M4 7h16" /></svg>
                    </div>
                    <h3 className="text-lg font-semibold text-gray-100">Destructive Forensics (Erasure)</h3>
                  </div>
                  <p className="text-gray-400 text-sm leading-relaxed font-mono">
                    Cryptographically wiping drives and overwriting raw storage to ensure sensitive data is permanently unrecoverable, complying with strict DoD sanitation protocols.
                  </p>
                </div>

                {/* Constructive Card */}
                <div className="bg-[#0a0a0c]/80 border border-blue-500/20 rounded-xl p-6 relative group hover:border-blue-500/50 transition-colors">
                  <div className="absolute top-0 left-0 w-full h-1 bg-gradient-to-r from-blue-600 to-transparent opacity-50"></div>
                  <div className="flex items-center mb-4">
                    <div className="w-10 h-10 rounded bg-blue-500/10 flex items-center justify-center mr-4 text-blue-500">
                      <svg xmlns="http://www.w3.org/2000/svg" className="h-6 w-6" fill="none" viewBox="0 0 24 24" stroke="currentColor"><path strokeLinecap="round" strokeLinejoin="round" strokeWidth={1.5} d="M21 21l-6-6m2-5a7 7 0 11-14 0 7 7 0 0114 0zM10 7v3m0 0v3m0-3h3m-3 0H7" /></svg>
                    </div>
                    <h3 className="text-lg font-semibold text-gray-100">Constructive Forensics (Carving)</h3>
                  </div>
                  <p className="text-gray-400 text-sm leading-relaxed font-mono">
                    Scanning heavily fragmented or damaged sectors to reconstruct critical digital artifacts and recover lost evidence from unallocated physical drive space.
                  </p>
                </div>
              </div>
            </div>
          </div>

          {/* Tech Stack Section */}
          <div className="mb-16">
            <h2 className="text-2xl font-semibold mb-6 flex items-center">
              <span className="w-2 h-2 bg-white rounded-full mr-3"></span>
              The Technology Stack
            </h2>
            <p className="text-gray-400 text-sm font-mono mb-8 max-w-3xl">
              Security and performance dictate our infrastructure. The platform is engineered using a highly optimized, cross-platform architecture allowing native hardware access.
            </p>

            <div className="grid grid-cols-1 md:grid-cols-2 gap-6">
              <div className="border border-gray-800 rounded-lg p-6 bg-gradient-to-br from-[#111115] to-[#0a0a0c]">
                <h3 className="text-sm font-mono text-gray-500 mb-2">FRONTEND CLIENT</h3>
                <p className="text-xl font-bold text-white mb-2">React + Tauri</p>
                <p className="text-sm text-gray-400">Delivering a lightweight, lightning-fast native desktop experience that consumes minimal system resources while maintaining a rich user interface.</p>
              </div>
              <div className="border border-gray-800 rounded-lg p-6 bg-gradient-to-br from-[#111115] to-[#0a0a0c]">
                <h3 className="text-sm font-mono text-gray-500 mb-2">BACKEND DAEMON</h3>
                <p className="text-xl font-bold text-white mb-2">FastAPI + Python Bridge</p>
                <p className="text-sm text-gray-400">Allowing our UI to seamlessly execute low-level system scripts and raw hardware interactions in a secure, isolated sandbox environment.</p>
              </div>
            </div>
          </div>

          {/* Team Section */}
          <div>
            <div className="flex flex-col md:flex-row md:items-end justify-between mb-8 border-b border-gray-800 pb-4">
              <div>
                <h2 className="text-2xl font-semibold mb-2 flex items-center">
                  <span className="w-2 h-2 bg-white rounded-full mr-3"></span>
                  Meet the Team
                </h2>
                <p className="text-gray-400 text-sm font-mono">
                  Developed for the Smart India Hackathon 2026.
                </p>
              </div>
              <div className="mt-4 md:mt-0 text-xs font-mono text-gray-500 bg-gray-900 px-3 py-1.5 rounded">
                6 ACTIVE MEMBERS
              </div>
            </div>

            <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-5">
              {teamMembers.map((member, idx) => (
                <div key={idx} className="bg-[#111115]/50 border border-gray-800 rounded-xl p-6 hover:border-gray-500 transition-colors group relative overflow-hidden">
                  
                  {/* Hover gradient effect */}
                  <div className="absolute inset-0 bg-gradient-to-br from-white/[0.02] to-transparent opacity-0 group-hover:opacity-100 transition-opacity"></div>
                  
                  <h3 className="text-xl font-bold text-white tracking-wide mb-1 relative z-10">{member.name}</h3>
                  <p className={`text-xs font-mono ${member.color} mb-4 uppercase relative z-10`}>{member.role}</p>
                  
                  <div className="w-full h-[1px] bg-gray-800 mb-4 relative z-10"></div>
                  
                  <p className="text-sm text-gray-400 leading-relaxed relative z-10">
                    {member.desc}
                  </p>
                </div>
              ))}
            </div>
          </div>

        </main>
      </div>
    </div>
  );
}