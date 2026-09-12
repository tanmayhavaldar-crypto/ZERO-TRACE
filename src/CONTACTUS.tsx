import React, { useState } from "react";

interface ContactUsProps {
  setCurrentView: (view: string) => void;
}

export default function CONTACTUS({ setCurrentView }: ContactUsProps) {
  // Form State
  const [formData, setFormData] = useState({
    name: "",
    role: "",
    type: "General",
    message: ""
  });
  const [isSubmitting, setIsSubmitting] = useState(false);
  const [submitStatus, setSubmitStatus] = useState<"idle" | "success" | "error">("idle");

  const handleChange = (e: React.ChangeEvent<HTMLInputElement | HTMLSelectElement | HTMLTextAreaElement>) => {
    setFormData({ ...formData, [e.target.name]: e.target.value });
  };

  const handleSubmit = (e: React.FormEvent) => {
    e.preventDefault();
    setIsSubmitting(true);
    
    // Simulate network request for the UI
    setTimeout(() => {
      setIsSubmitting(false);
      setSubmitStatus("success");
      setFormData({ name: "", role: "", type: "General", message: "" });
      
      // Reset success message after 5 seconds
      setTimeout(() => setSubmitStatus("idle"), 5000);
    }, 1500);
  };

  const developers = [
    { name: "Tanmay", role: "UI/UX & Integration Lead", email: "tanmayhavaldar@gmail.com", link: "[www.linkedin.com/in/tanmay-havaldar-a24878381]" },
    { name: "Krishna", role: "Frontend & Deployment Lead", email: "krishnamohanvidyarthi@gmail.com", link: "[www.linkedin.com/in/krishna4356169845168451]" },
    { name: "Rithik", role: "Hardware Wiping Specialist", email: "rajrithik401@gmail.com", link: "[linkedin.com/in/rithik-raj-b3b361387]" },
    { name: "Vidhatri", role: "File Sanitization Specialist", email: "Vidharti's Email", link: "linkedin.com/in/vidhatri-b-s-774568407" },
    { name: "Inchara", role: "File Carving Specialist", email: "Inchara's Email", link: "[Link]" },
    { name: "Ahan", role: "Drive Reconstruction Specialist", email: "ahankamal08@gmail.com", link: "[https://www.linkedin.com/in/ahankamal/]" },
  ];

  return (
    <div className="min-h-screen bg-[#0a0a0c] text-white font-sans relative overflow-x-hidden selection:bg-emerald-500/30 flex flex-col">
      
      {/* BACKGROUND LAYER */}
      <div className="absolute inset-0 z-0 pointer-events-none fixed">
        <img 
          src="https://images.unsplash.com/photo-1526374965328-7f61d4dc18c5?q=80&w=2070&auto=format&fit=crop" 
          alt="Cybersecurity" 
          className="w-full h-full object-cover opacity-10"
        />
        <div className="absolute inset-0 bg-gradient-to-r from-[#0a0a0c] via-[#0a0a0c]/90 to-[#0a0a0c]/80"></div>
        <div className="absolute inset-0 bg-gradient-to-b from-[#0a0a0c] via-transparent to-[#0a0a0c]/90"></div>
        {/* Emerald glow for Contact/Comms theme */}
        <div className="absolute top-[10%] right-[10%] w-[40%] h-[40%] bg-emerald-900/10 blur-[120px] rounded-full mix-blend-screen"></div>
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
              <button onClick={() => setCurrentView("forensic")} className="text-gray-400 hover:text-white transition cursor-pointer">Forensic History</button>
              <button onClick={() => setCurrentView("aboutus")} className="text-gray-400 hover:text-white transition cursor-pointer">About Us</button>
            </div>

            <div className="flex items-center space-x-4">
              <button className="px-5 py-2 text-sm font-mono border border-emerald-500/50 text-emerald-400 rounded hover:bg-emerald-500/10 shadow-[0_0_10px_rgba(16,185,129,0.2)] transition">
                Comms Link: SECURE
              </button>
            </div>
          </nav>
        </header>

        {/* Main Content Area */}
        <main className="flex-1 flex flex-col max-w-7xl mx-auto px-4 pt-12 w-full">
          
          {/* Header */}
          <div className="mb-12">
            <h1 className="text-4xl md:text-5xl font-semibold leading-tight tracking-tight mb-4 drop-shadow-lg">
              Contact & Support
            </h1>
            <p className="text-gray-400 text-sm md:text-base leading-relaxed font-mono max-w-3xl">
              Get in touch with the Z.E.R.O T.R.A.C.E. engineering team. Whether you are a forensic investigator testing our artifact carving engine, a cybersecurity professional auditing our zero-residue erasure protocols, or a hackathon mentor providing feedback, our secure channels are open.
            </p>
          </div>

          <div className="grid grid-cols-1 lg:grid-cols-5 gap-8">
            
            {/* LEFT COLUMN: Info Panels (Takes up 3/5 width on desktop) */}
            <div className="lg:col-span-3 space-y-8">
              
              {/* SIH Details Card */}
              <div className="bg-[#111115]/80 backdrop-blur-xl border border-white/10 rounded-2xl p-8 shadow-xl">
                <h2 className="text-xl font-medium text-gray-100 flex items-center border-b border-gray-800 pb-4 mb-6">
                  <span className="w-3 h-3 bg-white rounded-sm mr-3"></span>
                  Smart India Hackathon 2026 Details
                </h2>
                <p className="text-gray-400 text-sm font-mono mb-6">
                  For official SIH evaluation and repository access, please refer to the following project identifiers:
                </p>
                
                <div className="grid grid-cols-1 sm:grid-cols-2 gap-4">
                  <div className="bg-[#0a0a0c] border border-gray-800 p-4 rounded-lg">
                    <p className="text-xs text-gray-500 font-mono mb-1 uppercase">Team Name</p>
                    <p className="text-white font-medium">THETA</p>
                  </div>
                  <div className="bg-[#0a0a0c] border border-gray-800 p-4 rounded-lg">
                    <p className="text-xs text-gray-500 font-mono mb-1 uppercase">Problem Statement ID</p>
                    <p className="text-emerald-400 font-mono">SIH26149 / NTRO</p>
                  </div>
                  <div className="bg-[#0a0a0c] border border-gray-800 p-4 rounded-lg">
                    <p className="text-xs text-gray-500 font-mono mb-1 uppercase">Project Repository</p>
                    <a href="#" className="text-blue-400 hover:text-blue-300 underline underline-offset-4 text-sm break-all transition-colors">[Insert GitHub Link]</a>
                  </div>
                  <div className="bg-[#0a0a0c] border border-gray-800 p-4 rounded-lg">
                    <p className="text-xs text-gray-500 font-mono mb-1 uppercase">Documentation</p>
                    <a href="#" className="text-blue-400 hover:text-blue-300 underline underline-offset-4 text-sm break-all transition-colors">[Insert Link to Wiki/ReadMe]</a>
                  </div>
                </div>
              </div>

              {/* Developers Card */}
              <div className="bg-[#111115]/80 backdrop-blur-xl border border-white/10 rounded-2xl p-8 shadow-xl">
                <h2 className="text-xl font-medium text-gray-100 flex items-center border-b border-gray-800 pb-4 mb-6">
                  <svg xmlns="http://www.w3.org/2000/svg" className="h-6 w-6 mr-3 text-emerald-500" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                    <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={1.5} d="M17 20h5v-2a3 3 0 00-5.356-1.857M17 20H7m10 0v-2c0-.656-.126-1.283-.356-1.857M7 20H2v-2a3 3 0 015.356-1.857M7 20v-2c0-.656.126-1.283.356-1.857m0 0a5.002 5.002 0 019.288 0M15 7a3 3 0 11-6 0 3 3 0 016 0zm6 3a2 2 0 11-4 0 2 2 0 014 0zM7 10a2 2 0 11-4 0 2 2 0 014 0z" />
                  </svg>
                  Connect with the Developers
                </h2>
                <p className="text-gray-400 text-sm font-mono mb-6">
                  Reach out directly to our core engineers regarding specific system modules:
                </p>

                <div className="space-y-4">
                  {developers.map((dev, idx) => (
                    <div key={idx} className="flex flex-col sm:flex-row sm:items-center justify-between p-4 bg-[#0a0a0c]/50 border border-gray-800 rounded-lg hover:border-gray-600 transition-colors group">
                      <div className="mb-3 sm:mb-0">
                        <p className="text-white font-bold tracking-wide">{dev.name}</p>
                        <p className="text-xs font-mono text-gray-500 uppercase">{dev.role}</p>
                      </div>
                      <div className="flex flex-col sm:items-end text-sm font-mono space-y-1">
                        <a href={`mailto:${dev.email}`} className="text-gray-400 hover:text-emerald-400 transition-colors flex items-center">
                          <svg xmlns="http://www.w3.org/2000/svg" className="h-3 w-3 mr-1.5" viewBox="0 0 20 20" fill="currentColor"><path d="M2.003 5.884L10 9.882l7.997-3.998A2 2 0 0016 4H4a2 2 0 00-1.997 1.884z" /><path d="M18 8.118l-8 4-8-4V14a2 2 0 002 2h12a2 2 0 002-2V8.118z" /></svg>
                          {dev.email}
                        </a>
                        <a href={dev.link} className="text-gray-400 hover:text-blue-400 transition-colors flex items-center">
                          <svg xmlns="http://www.w3.org/2000/svg" className="h-3 w-3 mr-1.5" viewBox="0 0 20 20" fill="currentColor"><path fillRule="evenodd" d="M12.586 4.586a2 2 0 112.828 2.828l-3 3a2 2 0 01-2.828 0 1 1 0 00-1.414 1.414 4 4 0 005.656 0l3-3a4 4 0 00-5.656-5.656l-1.5 1.5a1 1 0 101.414 1.414l1.5-1.5zm-5 5a2 2 0 012.828 0 1 1 0 101.414-1.414 4 4 0 00-5.656 0l-3 3a4 4 0 105.656 5.656l1.5-1.5a1 1 0 10-1.414-1.414l-1.5 1.5a2 2 0 11-2.828-2.828l3-3z" clipRule="evenodd" /></svg>
                          LinkedIn Profile
                        </a>
                      </div>
                    </div>
                  ))}
                </div>
              </div>
            </div>

            {/* RIGHT COLUMN: Contact Form (Takes up 2/5 width on desktop) */}
            <div className="lg:col-span-2">
              <div className="bg-[#111115]/80 backdrop-blur-xl border border-white/10 rounded-2xl p-8 shadow-[0_0_40px_rgba(0,0,0,0.6)] h-full">
                <h2 className="text-xl font-medium text-gray-100 flex items-center border-b border-gray-800 pb-4 mb-6">
                  <span className="w-3 h-3 bg-emerald-500 rounded-sm mr-3 animate-pulse"></span>
                  Technical Inquiries
                </h2>
                
                {submitStatus === "success" ? (
                  <div className="h-64 flex flex-col items-center justify-center text-center animate-in fade-in zoom-in duration-500">
                    <div className="w-16 h-16 bg-emerald-500/20 rounded-full flex items-center justify-center mb-4">
                      <svg xmlns="http://www.w3.org/2000/svg" className="h-8 w-8 text-emerald-400" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                        <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M5 13l4 4L19 7" />
                      </svg>
                    </div>
                    <h3 className="text-lg font-bold text-white mb-2">Transmission Secure</h3>
                    <p className="text-gray-400 text-sm font-mono">Your inquiry has been encrypted and sent to the development team. We will review and respond shortly.</p>
                  </div>
                ) : (
                  <form onSubmit={handleSubmit} className="space-y-5">
                    <div>
                      <label className="block text-xs font-mono text-gray-400 mb-2 uppercase">Name</label>
                      <input 
                        type="text" 
                        name="name"
                        required
                        value={formData.name}
                        onChange={handleChange}
                        className="w-full bg-[#0a0a0c] border border-gray-700 text-white rounded-lg py-3 px-4 outline-none focus:border-emerald-500 focus:ring-1 focus:ring-emerald-500 transition-all font-mono text-sm"
                        placeholder="e.g. Dr. A. Sharma"
                      />
                    </div>

                    <div>
                      <label className="block text-xs font-mono text-gray-400 mb-2 uppercase">Organization / Role</label>
                      <input 
                        type="text" 
                        name="role"
                        required
                        value={formData.role}
                        onChange={handleChange}
                        className="w-full bg-[#0a0a0c] border border-gray-700 text-white rounded-lg py-3 px-4 outline-none focus:border-emerald-500 focus:ring-1 focus:ring-emerald-500 transition-all font-mono text-sm"
                        placeholder="e.g. SIH Mentor / NTRO Analyst"
                      />
                    </div>

                    <div>
                      <label className="block text-xs font-mono text-gray-400 mb-2 uppercase">Inquiry Type</label>
                      <select 
                        name="type"
                        value={formData.type}
                        onChange={handleChange}
                        className="w-full bg-[#0a0a0c] border border-gray-700 text-white rounded-lg py-3 px-4 outline-none focus:border-emerald-500 focus:ring-1 focus:ring-emerald-500 transition-all font-mono text-sm appearance-none"
                      >
                        <option value="Erasure Module">Erasure Module</option>
                        <option value="Carving Module">Carving Module</option>
                        <option value="UI Architecture">UI / Architecture</option>
                        <option value="General">General Inquiry</option>
                      </select>
                    </div>

                    <div>
                      <label className="block text-xs font-mono text-gray-400 mb-2 uppercase">Message</label>
                      <textarea 
                        name="message"
                        required
                        rows={4}
                        value={formData.message}
                        onChange={handleChange}
                        className="w-full bg-[#0a0a0c] border border-gray-700 text-white rounded-lg py-3 px-4 outline-none focus:border-emerald-500 focus:ring-1 focus:ring-emerald-500 transition-all font-mono text-sm resize-none"
                        placeholder="Enter your system feedback or inquiry here..."
                      ></textarea>
                    </div>

                    <button 
                      type="submit"
                      disabled={isSubmitting}
                      className="w-full bg-emerald-600/10 border border-emerald-500/50 text-emerald-500 hover:bg-emerald-600 hover:text-white py-3.5 rounded-lg transition-all shadow-[0_0_15px_rgba(16,185,129,0.15)] hover:shadow-[0_0_20px_rgba(16,185,129,0.4)] font-semibold text-sm disabled:opacity-50 disabled:cursor-not-allowed flex items-center justify-center cursor-pointer"
                    >
                      {isSubmitting ? (
                        <>
                          <svg className="animate-spin -ml-1 mr-3 h-4 w-4 text-current" xmlns="http://www.w3.org/2000/svg" fill="none" viewBox="0 0 24 24"><circle className="opacity-25" cx="12" cy="12" r="10" stroke="currentColor" strokeWidth="4"></circle><path className="opacity-75" fill="currentColor" d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4zm2 5.291A7.962 7.962 0 014 12H0c0 3.042 1.135 5.824 3 7.938l3-2.647z"></path></svg>
                          ENCRYPTING & SENDING...
                        </>
                      ) : (
                        "SUBMIT INQUIRY"
                      )}
                    </button>
                  </form>
                )}
              </div>
            </div>

          </div>
        </main>
      </div>
    </div>
  );
}