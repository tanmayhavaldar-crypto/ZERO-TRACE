import React, { useState } from "react";

interface RecoverToolProps {
  setCurrentView: (view: string) => void;
}

interface RecoveredFile {
  id: string;
  name: string;
  type: string;
  size: string;
  health: "High" | "Medium" | "Corrupted";
}

export default function RECOVERTOOL({ setCurrentView }: RecoverToolProps) {
  const [isScanning, setIsScanning] = useState(false);
  const [scanProgress, setScanProgress] = useState(0);
  const [scanLogs, setScanLogs] = useState<string[]>([
    "[SYSTEM] Awaiting root-access initialization...",
    "[SYSTEM] Target: PhysicalDrive0 (Backend Daemon Link: STANDBY)",
  ]);
  const [recoveredFiles, setRecoveredFiles] = useState<RecoveredFile[]>([]);
  const [isRestoring, setIsRestoring] = useState(false);

  // Simulate Root-Level Deep Scan
  const initiateDeepScan = () => {
    setIsScanning(true);
    setScanProgress(0);
    setRecoveredFiles([]);
    setScanLogs([
      "[SYSTEM] Link established with NTRO Backend Daemon.",
      "[FORENSIC] Initiating Root-Level Sector Bypass...",
    ]);

    let progress = 0;
    const interval = setInterval(() => {
      progress += Math.floor(Math.random() * 8) + 2;

      if (progress >= 100) {
        progress = 100;
        clearInterval(interval);
        setIsScanning(false);
        setScanLogs((prev) => [
          ...prev,
          "[FORENSIC] MFT parsing complete.",
          "[FORENSIC] Data carving successful.",
          "[SUCCESS] 5 hidden/deleted fragments reconstructed.",
        ]);
        
        // Populate fake recovered data
        setRecoveredFiles([
          { id: "0x1A4F", name: "auth_keys.pgp", type: "Encryption", size: "4.2 KB", health: "High" },
          { id: "0x2B8C", name: "sys_log_2023.dmp", type: "System", size: "142 MB", health: "Medium" },
          { id: "0x9F01", name: "db_backup_shadow.sql", type: "Database", size: "890 MB", health: "High" },
          { id: "0x3C44", name: "cache_fragment.bin", type: "Raw Data", size: "12 KB", health: "Corrupted" },
          { id: "0x7D2A", name: "deleted_image.jpg", type: "Media", size: "3.4 MB", health: "High" },
        ]);
      } else {
        const actions = [
          "Bypassing OS API hooks...",
          "Reading raw volume shadow copy...",
          "Carving file headers (JPEG, SQL, PGP)...",
          "Analyzing unallocated slack space...",
          `Scanning sector 0x00${Math.floor(Math.random() * 99999)}...`
        ];
        const randomAction = actions[Math.floor(Math.random() * actions.length)];
        setScanLogs((prev) => [...prev, `[SCANNING] ${randomAction}`]);
      }
      setScanProgress(progress);
    }, 500);
  };

  const handleRestore = () => {
    setIsRestoring(true);
    setScanLogs((prev) => [...prev, "[WARNING] Extracting selected files to secure backend server..."]);
    setTimeout(() => {
      setIsRestoring(false);
      setScanLogs((prev) => [...prev, "[SUCCESS] Files successfully decrypted and moved to Secure Vault."]);
      setRecoveredFiles([]);
    }, 3000);
  };

  return (
    <div className="min-h-screen bg-[#0a0a0c] text-white font-sans relative overflow-x-hidden selection:bg-red-500/30 flex flex-col">
      
      {/* BACKGROUND LAYER */}
      <div className="absolute inset-0 z-0 pointer-events-none fixed">
        <img 
          src="https://images.unsplash.com/photo-1526374965328-7f61d4dc18c5?q=80&w=2070&auto=format&fit=crop" 
          alt="Cybersecurity" 
          className="w-full h-full object-cover opacity-20"
        />
        <div className="absolute inset-0 bg-gradient-to-r from-[#0a0a0c] via-[#0a0a0c]/80 to-transparent"></div>
        <div className="absolute inset-0 bg-gradient-to-b from-[#0a0a0c] via-transparent to-[#0a0a0c]/90"></div>
        {/* Blue/Purple ambient glow for the Recovery theme */}
        <div className="absolute top-[-10%] left-[-10%] w-[50%] h-[50%] bg-blue-900/20 blur-[120px] rounded-full mix-blend-screen"></div>
        <div className="absolute bottom-[-10%] right-[-10%] w-[50%] h-[50%] bg-purple-900/20 blur-[120px] rounded-full mix-blend-screen"></div>
      </div>

      {/* FOREGROUND CONTENT LAYER */}
      <div className="relative z-10 flex flex-col flex-1">
        
        {/* Navigation */}
        <header className="border-b border-white/5 bg-[#0a0a0c]/80 backdrop-blur-md sticky top-0 z-40">
          <nav className="flex items-center justify-between px-8 py-5 max-w-7xl w-full mx-auto">
            
            <div 
              className="text-2xl font-black tracking-tighter cursor-pointer"
              onClick={() => setCurrentView("home")}
            >
              ZERO TRACE<span className="text-red-500">.</span>
            </div>

            <div className="hidden md:flex items-center space-x-8 text-sm font-medium">
              <button 
                onClick={() => setCurrentView("erasing")} 
                className="text-gray-400 hover:text-white transition cursor-pointer"
              >
                Erasing Tool
              </button>
              
              <button className="text-blue-500 relative cursor-pointer">
                Recover Tool
                <span className="absolute -bottom-6 left-0 w-full h-[2px] bg-blue-500"></span>
              </button>

             {/* UPDATED: Navigates to FORENSIC HISTORY */}
              <button 
                onClick={() => setCurrentView("forensic")} 
                className="text-gray-400 hover:text-white transition cursor-pointer"
              >
                Forensic History
              </button>
              <button onClick={() => setCurrentView("aboutus")} className="text-gray-400 hover:text-white transition cursor-pointer">About Us</button>

            </div>

            <div className="flex items-center space-x-4">
              <button className="px-5 py-2 text-sm font-medium border border-blue-500/50 text-blue-400 rounded hover:bg-blue-500/10 transition ">
                Backend Link: ONLINE
              </button>
            </div>
          </nav>
        </header>

        {/* Main Content Area */}
        <main className="flex-1 flex flex-col items-center max-w-7xl mx-auto px-4 py-12 w-full">
          
          <div className="text-center max-w-4xl mb-10 px-4">
            <h1 className="text-4xl md:text-5xl font-semibold leading-tight tracking-tight mb-4 drop-shadow-lg">
              Forensic Data Recovery
            </h1>
            <p className="text-gray-400 text-sm md:text-base leading-relaxed font-mono max-w-2xl mx-auto">
              Command the backend daemon to scan physical drives for deleted fragments. Recovered data will be pulled from unallocated space and restored to the secure server.
            </p>
          </div>

          <div className="w-full max-w-6xl grid grid-cols-1 lg:grid-cols-2 gap-8 bg-[#111115]/80 backdrop-blur-xl border border-white/10 rounded-2xl p-6 md:p-8 shadow-[0_0_40px_rgba(0,0,0,0.8)]">
            
            {/* LEFT COLUMN: Controls & Terminal */}
            <div className="flex flex-col space-y-6">
              <h2 className="text-xl font-medium text-gray-100 flex items-center border-b border-gray-800 pb-3">
                <span className="w-3 h-3 bg-blue-500 rounded-sm mr-3 animate-pulse"></span>
                1. System Scanner
              </h2>

              <button 
                onClick={initiateDeepScan}
                disabled={isScanning || scanProgress === 100}
                className="w-full bg-blue-600/10 border border-blue-500/50 text-blue-400 py-6 rounded-xl hover:bg-blue-600/20 disabled:opacity-50 disabled:cursor-not-allowed transition font-semibold text-lg group flex flex-col items-center justify-center"
              >
                <svg xmlns="http://www.w3.org/2000/svg" className="h-8 w-8 mb-2 group-hover:scale-110 transition-transform" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                  <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={1.5} d="M21 21l-6-6m2-5a7 7 0 11-14 0 7 7 0 0114 0zM10 7v3m0 0v3m0-3h3m-3 0H7" />
                </svg>
                {isScanning ? "SCAN IN PROGRESS..." : "INITIATE ROOT-LEVEL DEEP SCAN"}
              </button>

              <div className="w-full bg-gray-900 rounded-full h-2.5 border border-gray-800 mt-2">
                <div 
                  className="bg-blue-500 h-2.5 rounded-full transition-all duration-300 relative overflow-hidden" 
                  style={{ width: `${scanProgress}%` }}
                >
                  {isScanning && <div className="absolute inset-0 bg-white/20 w-full animate-[pulse_1s_ease-in-out_infinite]"></div>}
                </div>
              </div>

              <div className="bg-[#050505] border border-gray-800 rounded-lg p-4 h-64 flex flex-col relative overflow-hidden group mt-4">
                <div className="absolute top-0 left-0 w-full h-1 bg-gradient-to-r from-blue-500/20 via-purple-500/20 to-transparent"></div>
                <h3 className="text-xs font-mono text-gray-600 mb-2 border-b border-gray-800/50 pb-1">TERMINAL // NTRO-DAEMON</h3>
                
                <div className="overflow-y-auto flex-1 font-mono text-[13px] space-y-1">
                  {scanLogs.map((log, idx) => (
                    <p key={idx} className={`${
                      log.includes('[WARNING]') ? 'text-orange-400' :
                      log.includes('[SUCCESS]') ? 'text-green-400' :
                      log.includes('[FORENSIC]') ? 'text-purple-400' : 
                      log.includes('[SCANNING]') ? 'text-blue-400' : 'text-gray-400'
                    }`}>
                      {log}
                    </p>
                  ))}
                  {isScanning && <span className="inline-block w-2 h-4 bg-gray-400 animate-pulse mt-1"></span>}
                </div>
              </div>
            </div>

            {/* RIGHT COLUMN: Recovered Vault */}
            <div className="flex flex-col space-y-6">
              <h2 className="text-xl font-medium text-gray-100 flex items-center border-b border-gray-800 pb-3">
                <span className="w-3 h-3 bg-purple-500 rounded-sm mr-3"></span>
                2. Data Vault
              </h2>

              <div className="bg-[#0a0a0c] border border-gray-800 rounded-lg flex-1 overflow-hidden flex flex-col h-[320px]">
                {recoveredFiles.length === 0 ? (
                  <div className="flex-1 flex flex-col items-center justify-center text-gray-600 font-mono text-sm p-6 text-center">
                    <svg xmlns="http://www.w3.org/2000/svg" className="h-12 w-12 mb-4 opacity-50" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                      <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={1} d="M19 11H5m14 0a2 2 0 012 2v6a2 2 0 01-2 2H5a2 2 0 01-2-2v-6a2 2 0 012-2m14 0V9a2 2 0 00-2-2M5 11V9a2 2 0 012-2m0 0V5a2 2 0 012-2h6a2 2 0 012 2v2M7 7h10" />
                    </svg>
                    <p>Vault is empty.</p>
                    <p className="mt-2 opacity-70">Run Deep Scan to carve hidden or deleted files from the drive.</p>
                  </div>
                ) : (
                  <div className="overflow-y-auto">
                    <table className="w-full text-left font-mono text-sm border-collapse">
                      <thead className="bg-gray-900/50 text-gray-400 text-xs uppercase sticky top-0">
                        <tr>
                          <th className="px-4 py-3 font-medium">File / Hash</th>
                          <th className="px-4 py-3 font-medium">Size</th>
                          <th className="px-4 py-3 font-medium">Integrity</th>
                        </tr>
                      </thead>
                      <tbody className="divide-y divide-gray-800">
                        {recoveredFiles.map((file) => (
                          <tr key={file.id} className="hover:bg-white/[0.02] transition-colors">
                            <td className="px-4 py-3">
                              <p className="text-gray-200">{file.name}</p>
                              <p className="text-gray-600 text-xs">{file.id}</p>
                            </td>
                            <td className="px-4 py-3 text-gray-400">{file.size}</td>
                            <td className="px-4 py-3">
                              <span className={`px-2 py-1 rounded text-xs font-semibold ${
                                file.health === "High" ? "bg-green-500/10 text-green-400" :
                                file.health === "Medium" ? "bg-orange-500/10 text-orange-400" :
                                "bg-red-500/10 text-red-400"
                              }`}>
                                {file.health}
                              </span>
                            </td>
                          </tr>
                        ))}
                      </tbody>
                    </table>
                  </div>
                )}
              </div>

              <button 
                onClick={handleRestore}
                disabled={recoveredFiles.length === 0 || isRestoring}
                className="w-full bg-white text-black py-4 rounded hover:bg-gray-200 disabled:opacity-50 disabled:cursor-not-allowed transition shadow-[0_0_15px_rgba(255,255,255,0.2)] font-semibold flex items-center justify-center"
              >
                {isRestoring ? (
                  <span className="flex items-center">
                    <svg className="animate-spin -ml-1 mr-3 h-5 w-5 text-black" xmlns="http://www.w3.org/2000/svg" fill="none" viewBox="0 0 24 24"><circle className="opacity-25" cx="12" cy="12" r="10" stroke="currentColor" strokeWidth="4"></circle><path className="opacity-75" fill="currentColor" d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4zm2 5.291A7.962 7.962 0 014 12H0c0 3.042 1.135 5.824 3 7.938l3-2.647z"></path></svg>
                    Extracting to Backend...
                  </span>
                ) : (
                  "EXTRACT RECOVERED DATA"
                )}
              </button>
            </div>
            
          </div>
        </main>
      </div>
    </div>
  );
}