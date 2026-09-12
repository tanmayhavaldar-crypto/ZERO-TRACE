import React, { useState } from "react";

interface ForensicHistoryProps {
  setCurrentView: (view: string) => void;
}

interface LogEntry {
  id: string;
  timestamp: string;
  operation: string;
  target: string;
  status: "SUCCESS" | "FAILED" | "INTERRUPTED";
}

export default function FORENSICHISTORY({ setCurrentView }: ForensicHistoryProps) {
  // Security State
  const [masterPassword, setMasterPassword] = useState("12345678");
  const [authModalOpen, setAuthModalOpen] = useState(false);
  const [authMode, setAuthMode] = useState<"clear_history" | "change_password" | null>(null);
  
  // Form State
  const [inputPassword, setInputPassword] = useState("");
  const [newPassword, setNewPassword] = useState("");
  const [authError, setAuthError] = useState("");
  const [systemMessage, setSystemMessage] = useState("");

  // Dummy Forensic Data
  const [historyLogs, setHistoryLogs] = useState<LogEntry[]>([
    { id: "LOG-9281", timestamp: "2026-09-03 14:32:01", operation: "DoD 5220.22-M Wipe", target: "PhysicalDrive1", status: "SUCCESS" },
    { id: "LOG-9280", timestamp: "2026-09-02 09:15:22", operation: "Deep Sector Scan", target: "C:/Users/Admin/AppData", status: "SUCCESS" },
    { id: "LOG-9279", timestamp: "2026-08-30 22:01:45", operation: "MFT Reconstruction", target: "Volume Shadow Copy", status: "FAILED" },
    { id: "LOG-9278", timestamp: "2026-08-28 11:44:10", operation: "Zero-Fill Erase", target: "E:/Confidential_2025", status: "SUCCESS" },
    { id: "LOG-9277", timestamp: "2026-08-28 11:40:05", operation: "Target Selection", target: "E:/Confidential_2025", status: "INTERRUPTED" },
  ]);

  // Handle Modal Open
  const openAuthModal = (mode: "clear_history" | "change_password") => {
    setAuthMode(mode);
    setInputPassword("");
    setNewPassword("");
    setAuthError("");
    setAuthModalOpen(true);
  };

  // Handle Authentication Submission
  const handleAuthSubmit = (e: React.FormEvent) => {
    e.preventDefault();
    
    if (inputPassword !== masterPassword) {
      setAuthError("CRITICAL: Invalid credentials provided.");
      return;
    }

    if (authMode === "clear_history") {
      setHistoryLogs([]);
      setSystemMessage("[SYS] Forensic history permanently purged.");
      setAuthModalOpen(false);
    } else if (authMode === "change_password") {
      if (newPassword.length < 4) {
        setAuthError("New PIN must be at least 4 characters.");
        return;
      }
      setMasterPassword(newPassword);
      setSystemMessage("[SYS] Master PIN successfully updated.");
      setAuthModalOpen(false);
    }
    
    // Clear system message after 4 seconds
    setTimeout(() => setSystemMessage(""), 4000);
  };

  return (
    <div className="min-h-screen bg-[#0a0a0c] text-white font-sans relative overflow-x-hidden selection:bg-amber-500/30 flex flex-col">
      
      {/* BACKGROUND LAYER */}
      <div className="absolute inset-0 z-0 pointer-events-none fixed">
        <img 
          src="https://images.unsplash.com/photo-1526374965328-7f61d4dc18c5?q=80&w=2070&auto=format&fit=crop" 
          alt="Cybersecurity" 
          className="w-full h-full object-cover opacity-20"
        />
        <div className="absolute inset-0 bg-gradient-to-r from-[#0a0a0c] via-[#0a0a0c]/90 to-[#0a0a0c]/80"></div>
        <div className="absolute inset-0 bg-gradient-to-b from-[#0a0a0c] via-transparent to-[#0a0a0c]/90"></div>
        {/* Amber glow for the History/Audit theme */}
        <div className="absolute top-[-10%] left-[20%] w-[40%] h-[50%] bg-amber-900/10 blur-[120px] rounded-full mix-blend-screen"></div>
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
              <button onClick={() => setCurrentView("erasing")} className="text-gray-400 hover:text-white transition cursor-pointer">Erasing Tool</button>
              <button onClick={() => setCurrentView("recover")} className="text-gray-400 hover:text-white transition cursor-pointer">Recover Tool</button>
              <button className="text-amber-500 relative cursor-pointer ">
                Forensic History
                <span className="absolute -bottom-6 left-0 w-full h-[2px] bg-amber-500"></span>
              </button>
              <button onClick={() => setCurrentView("aboutus")} className="text-gray-400 hover:text-white transition cursor-pointer">About Us</button>

            </div>

            <div className="flex items-center space-x-4">
              <button className="px-5 py-2 text-sm font-mono border border-amber-500/50 text-amber-400 rounded hover:bg-amber-500/5 transition  ">
                Audit Status: LOGGING
              </button>
            </div>
          </nav>
        </header>

        {/* Main Content Area */}
        <main className="flex-1 flex flex-col max-w-7xl mx-auto px-4 py-10 w-full">
          
          <div className="mb-8">
            <h1 className="text-4xl md:text-5xl font-semibold leading-tight tracking-tight mb-2 drop-shadow-lg">
              System Audit Logs
            </h1>
            <p className="text-gray-400 text-sm font-mono">
              Immutable record of all sanitization and extraction operations performed by the ZERO TRACE core.
            </p>
          </div>

          {systemMessage && (
            <div className="mb-6 p-4 bg-green-500/10 border border-green-500/50 text-green-400 font-mono text-sm rounded-lg animate-pulse">
              {systemMessage}
            </div>
          )}

          <div className="w-full grid grid-cols-1 lg:grid-cols-3 gap-8">
            
            {/* LEFT COLUMN: Data Table (Spans 2 columns) */}
            <div className="lg:col-span-2 bg-[#111115]/80 backdrop-blur-xl border border-white/10 rounded-2xl shadow-[0_0_40px_rgba(0,0,0,0.8)] overflow-hidden flex flex-col h-[500px]">
              <div className="p-5 border-b border-gray-800 bg-white/[0.02] flex justify-between items-center">
                <h2 className="text-lg font-medium text-gray-100 flex items-center">
                  <svg xmlns="http://www.w3.org/2000/svg" className="h-5 w-5 mr-2 text-amber-500" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                    <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M9 12h6m-6 4h6m2 5H7a2 2 0 01-2-2V5a2 2 0 012-2h5.586a1 1 0 01.707.293l5.414 5.414a1 1 0 01.293.707V19a2 2 0 01-2 2z" />
                  </svg>
                  Forensic Event History
                </h2>
                <span className="text-xs font-mono text-gray-500">TOTAL ENTRIES: {historyLogs.length}</span>
              </div>
              
              <div className="flex-1 overflow-y-auto">
                {historyLogs.length === 0 ? (
                  <div className="h-full flex flex-col items-center justify-center text-gray-600 font-mono text-sm">
                    <svg xmlns="http://www.w3.org/2000/svg" className="h-12 w-12 mb-3 opacity-30" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                      <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={1} d="M19 7l-.867 12.142A2 2 0 0116.138 21H7.862a2 2 0 01-1.995-1.858L5 7m5 4v6m4-6v6m1-10V4a1 1 0 00-1-1h-4a1 1 0 00-1 1v3M4 7h16" />
                    </svg>
                    <p>No forensic logs found.</p>
                  </div>
                ) : (
                  <table className="w-full text-left font-mono text-sm border-collapse">
                    <thead className="bg-[#0a0a0c] text-gray-500 text-xs uppercase sticky top-0 z-10 shadow-md">
                      <tr>
                        <th className="px-6 py-4 font-medium">Timestamp</th>
                        <th className="px-6 py-4 font-medium">Operation</th>
                        <th className="px-6 py-4 font-medium">Target</th>
                        <th className="px-6 py-4 font-medium text-right">Status</th>
                      </tr>
                    </thead>
                    <tbody className="divide-y divide-gray-800/50">
                      {historyLogs.map((log, index) => (
                        <tr key={index} className="hover:bg-white/[0.02] transition-colors group">
                          <td className="px-6 py-4 text-gray-400 whitespace-nowrap">
                            <span className="block text-gray-600 text-[10px] mb-0.5">{log.id}</span>
                            {log.timestamp}
                          </td>
                          <td className="px-6 py-4 text-gray-200">{log.operation}</td>
                          <td className="px-6 py-4 text-gray-400 truncate max-w-[150px]" title={log.target}>{log.target}</td>
                          <td className="px-6 py-4 text-right">
                            <span className={`inline-flex items-center px-2.5 py-0.5 rounded-full text-xs font-medium border ${
                              log.status === "SUCCESS" ? "bg-green-500/10 text-green-400 border-green-500/20" :
                              log.status === "FAILED" ? "bg-red-500/10 text-red-400 border-red-500/20" :
                              "bg-gray-500/10 text-gray-400 border-gray-500/20"
                            }`}>
                              {log.status}
                            </span>
                          </td>
                        </tr>
                      ))}
                    </tbody>
                  </table>
                )}
              </div>
            </div>

            {/* RIGHT COLUMN: Security Controls */}
            <div className="bg-[#111115]/80 backdrop-blur-xl border border-white/10 rounded-2xl shadow-2xl p-6 h-fit">
              <h2 className="text-xl font-medium text-gray-100 flex items-center border-b border-gray-800 pb-4 mb-6">
                <svg xmlns="http://www.w3.org/2000/svg" className="h-6 w-6 mr-3 text-red-500" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                  <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={1.5} d="M12 15v2m-6 4h12a2 2 0 002-2v-6a2 2 0 00-2-2H6a2 2 0 00-2 2v6a2 2 0 002 2zm10-10V7a4 4 0 00-8 0v4h8z" />
                </svg>
                Security Controls
              </h2>

              <div className="space-y-4">
                <div className="p-4 rounded-xl border border-gray-800 bg-[#0a0a0c]/50">
                  <h3 className="text-sm font-medium text-gray-200 mb-1">Purge Records</h3>
                  <p className="text-xs text-gray-500 mb-4">Permanently delete all forensic history from this device. Requires Master PIN.</p>
                  <button 
                    onClick={() => openAuthModal("clear_history")}
                    disabled={historyLogs.length === 0}
                    className="w-full bg-red-600/10 border border-red-500/50 text-red-500 py-2.5 rounded hover:bg-red-600 hover:text-white disabled:opacity-30 disabled:cursor-not-allowed transition font-semibold text-sm cursor-pointer"
                  >
                    CLEAR FORENSIC LOGS
                  </button>
                </div>

                <div className="p-4 rounded-xl border border-gray-800 bg-[#0a0a0c]/50">
                  <h3 className="text-sm font-medium text-gray-200 mb-1">Authentication</h3>
                  <p className="text-xs text-gray-500 mb-4 ">Change the Master PIN required for administrative actions.</p>
                  <button 
                    onClick={() => openAuthModal("change_password")}
                    className="w-full bg-gray-800 border border-gray-700 text-gray-300 py-2.5 rounded hover:bg-gray-700 hover:text-white transition font-semibold text-sm cursor-pointer"
                  >
                    CHANGE MASTER PIN
                  </button>
                </div>
              </div>
            </div>

          </div>
        </main>
      </div>

      {/* =========================================
          AUTHENTICATION MODAL
          ========================================= */}
      {authModalOpen && (
        <div className="fixed inset-0 z-50 flex items-center justify-center p-4">
          {/* Backdrop */}
          <div 
            className="absolute inset-0 bg-black/80 backdrop-blur-sm"
            onClick={() => setAuthModalOpen(false)}
          ></div>
          
          {/* Modal Content */}
          <div className="relative bg-[#111115] border border-gray-700 rounded-xl shadow-2xl p-8 w-full max-w-md transform transition-all">
            <div className="flex items-center mb-6">
              <div className="w-10 h-10 rounded-full bg-red-500/10 flex items-center justify-center mr-4">
                <svg xmlns="http://www.w3.org/2000/svg" className="h-6 w-6 text-red-500" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                  <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M12 9v2m0 4h.01m-6.938 4h13.856c1.54 0 2.502-1.667 1.732-3L13.732 4c-.77-1.333-2.694-1.333-3.464 0L3.34 16c-.77 1.333.192 3 1.732 3z" />
                </svg>
              </div>
              <div>
                <h3 className="text-xl font-medium text-white">
                  {authMode === "clear_history" ? "Confirm Purge" : "Update Credentials"}
                </h3>
                <p className="text-sm text-gray-400 font-mono">Authorization required.</p>
              </div>
            </div>

            <form onSubmit={handleAuthSubmit} className="space-y-5">
              <div>
                <label className="block text-xs font-mono text-gray-400 mb-2 uppercase">Current Master PIN</label>
                <input 
                  type="password" 
                  value={inputPassword}
                  onChange={(e) => setInputPassword(e.target.value)}
                  className="w-full bg-[#0a0a0c] border border-gray-700 text-white rounded py-3 px-4 outline-none focus:border-red-500 transition-colors font-mono tracking-widest"
                  placeholder="••••••••"
                  autoFocus
                />
              </div>

              {authMode === "change_password" && (
                <div>
                  <label className="block text-xs font-mono text-gray-400 mb-2 uppercase">New Master PIN</label>
                  <input 
                    type="password" 
                    value={newPassword}
                    onChange={(e) => setNewPassword(e.target.value)}
                    className="w-full bg-[#0a0a0c] border border-gray-700 text-white rounded py-3 px-4 outline-none focus:border-red-500 transition-colors font-mono tracking-widest"
                    placeholder="••••••••"
                  />
                </div>
              )}

              {authError && (
                <p className="text-red-500 text-xs font-mono mt-2 flex items-center">
                  <span className="w-1.5 h-1.5 bg-red-500 rounded-full mr-2"></span>
                  {authError}
                </p>
              )}

              <div className="flex space-x-3 pt-4">
                <button 
                  type="button"
                  onClick={() => setAuthModalOpen(false)}
                  className="flex-1 bg-gray-800 text-white py-3 rounded hover:bg-gray-700 transition font-semibold text-sm cursor-pointer"
                >
                  CANCEL
                </button>
                <button 
                  type="submit"
                  className="flex-1 bg-red-600 text-white py-3 rounded hover:bg-red-500 transition shadow-[0_0_15px_rgba(220,38,38,0.3)] font-semibold text-sm cursor-pointer"
                >
                  {authMode === "clear_history" ? "EXECUTE PURGE" : "SAVE PIN"}
                </button>
              </div>
            </form>
          </div>
        </div>
      )}

    </div>
  );
}