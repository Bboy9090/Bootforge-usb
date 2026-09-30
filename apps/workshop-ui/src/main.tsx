import React, { useEffect, useMemo, useState } from "react";
import { createRoot } from "react-dom/client";
import { invoke } from "@tauri-apps/api/tauri";
import "./styles.css";

type DeviceMode = "Normal" | "Recovery" | "Dfu" | "Bootloader" | "Fastboot" | "Adb" | "MassStorage" | "Unknown";
type ViewMode = "devices" | "media";
type HostPlatform = "Windows" | "MacOs" | "Linux" | "Unknown";
type CapabilityLevel = "Supported" | "SupportedWithNativeBackend" | "PreparationOnly" | "Unsupported";
type MediaTarget = "WindowsInstaller" | "LinuxLive" | "MacOsOfficialInstaller" | "MacOsRawImage";

interface DeviceInfo {
  bus_number: number;
  address: number;
  vendor_id: number;
  product_id: number;
  vendor_name?: string | null;
  manufacturer?: string | null;
  product_name?: string | null;
  serial_number?: string | null;
  platform: string;
  transport: string;
  mode: DeviceMode;
  recommended_workflow?: string;
}

interface MediaCapability {
  host: HostPlatform;
  target: MediaTarget;
  level: CapabilityLevel;
  backend?: string | null;
  notes: string;
}

const demoDevices: DeviceInfo[] = [{
  bus_number: 1, address: 4, vendor_id: 0x05ac, product_id: 0x12a8,
  vendor_name: "Apple", manufacturer: "Apple Inc.", product_name: "iPhone",
  serial_number: "DEMO-DEVICE", platform: "Apple", transport: "Usb3",
  mode: "Normal", recommended_workflow: "StandardInspection",
}];

const demoMatrix: MediaCapability[] = [
  { host: "MacOs", target: "WindowsInstaller", level: "SupportedWithNativeBackend", backend: "macos-windows-media", notes: "Prepare Windows installer media from macOS." },
  { host: "MacOs", target: "LinuxLive", level: "SupportedWithNativeBackend", backend: "macos-raw-image", notes: "Write validated Linux hybrid/raw images." },
  { host: "MacOs", target: "MacOsOfficialInstaller", level: "SupportedWithNativeBackend", backend: "macos-createinstallmedia", notes: "Use Apple's supported createinstallmedia workflow." },
  { host: "MacOs", target: "MacOsRawImage", level: "SupportedWithNativeBackend", backend: "macos-raw-image", notes: "Restore compatible user-supplied raw images." },
];

const hex = (value: number) => value.toString(16).padStart(4, "0").toUpperCase();
const isDesktopRuntime = () => "__TAURI__" in window;

function targetLabel(target: MediaTarget) {
  switch (target) {
    case "WindowsInstaller": return "Windows installer USB";
    case "LinuxLive": return "Linux live USB";
    case "MacOsOfficialInstaller": return "Official macOS installer";
    case "MacOsRawImage": return "macOS raw image";
  }
}

function levelLabel(level: CapabilityLevel) {
  switch (level) {
    case "Supported": return "Supported";
    case "SupportedWithNativeBackend": return "Native backend";
    case "PreparationOnly": return "Preparation only";
    case "Unsupported": return "Unsupported";
  }
}

function App() {
  const [view, setView] = useState<ViewMode>("devices");
  const [devices, setDevices] = useState<DeviceInfo[]>([]);
  const [selected, setSelected] = useState<number | null>(null);
  const [scanning, setScanning] = useState(false);
  const [message, setMessage] = useState("Ready for a read-only USB scan.");
  const [host, setHost] = useState<HostPlatform>("Unknown");
  const [matrix, setMatrix] = useState<MediaCapability[]>([]);

  const active = selected === null ? undefined : devices[selected];
  const recoveryCount = useMemo(
    () => devices.filter((device) => device.mode !== "Normal" && device.mode !== "Unknown").length,
    [devices],
  );

  useEffect(() => {
    async function loadCapabilities() {
      if (!isDesktopRuntime()) {
        setHost("MacOs");
        setMatrix(demoMatrix);
        return;
      }
      try {
        const [runtimeHost, capabilities] = await Promise.all([
          invoke<HostPlatform>("current_host_platform"),
          invoke<MediaCapability[]>("media_capability_matrix"),
        ]);
        setHost(runtimeHost);
        setMatrix(capabilities);
      } catch (error) {
        setMessage(`Media capability query unavailable: ${String(error)}`);
      }
    }
    loadCapabilities();
  }, []);

  async function scan() {
    setScanning(true);
    setMessage("Inspecting USB descriptors…");
    try {
      const result = isDesktopRuntime() ? await invoke<DeviceInfo[]>("scan_connected_devices") : demoDevices;
      setDevices(result);
      setSelected(result.length > 0 ? 0 : null);
      setMessage(isDesktopRuntime()
        ? `${result.length} connected device${result.length === 1 ? "" : "s"} detected.`
        : "Preview mode: showing a safe sample device.");
    } catch (error) {
      setDevices([]);
      setSelected(null);
      setMessage(`Scan unavailable: ${String(error)}`);
    } finally {
      setScanning(false);
    }
  }

  return (
    <main className="app-shell">
      <aside className="sidebar">
        <div className="brand-mark" aria-hidden="true">B</div>
        <div className="brand-copy"><span>BootForge</span><small>cross-platform media studio</small></div>
        <nav aria-label="Primary">
          <button className={`nav-item ${view === "devices" ? "active" : ""}`} onClick={() => setView("devices")}><span>⌁</span> Device Forge</button>
          <button className={`nav-item ${view === "media" ? "active" : ""}`} onClick={() => setView("media")}><span>◇</span> Media Builder</button>
          <button className="nav-item" disabled><span>↻</span> Recovery Center</button>
          <button className="nav-item" disabled><span>▦</span> Session History</button>
        </nav>
        <div className="safety-card">
          <strong>Shared engine, native backends</strong>
          <p>Windows, macOS, and Linux use the same planner. Destructive writes remain behind host-specific gated backends.</p>
        </div>
        <footer>Windows · macOS · Linux</footer>
      </aside>

      <section className="workspace">
        {view === "devices" ? (
          <>
            <header className="topbar">
              <div><p className="eyebrow">DEVICE FORGE</p><h1>Know what is connected before you act.</h1></div>
              <div className="runtime-pill"><i className={isDesktopRuntime() ? "online" : "preview"} />{isDesktopRuntime() ? `${host} host` : "Browser preview"}</div>
            </header>

            <div className="hero-panel">
              <div><span className="status-label">FORGE STATUS</span><h2>{scanning ? "Reading the signal…" : "BootForge is standing by."}</h2><p>{message}</p></div>
              <button className="scan-button" onClick={scan} disabled={scanning}>{scanning ? "Scanning…" : "Scan USB Devices"}</button>
            </div>

            <div className="metric-grid">
              <Metric label="Connected" value={devices.length.toString()} detail="visible USB devices" />
              <Metric label="Special modes" value={recoveryCount.toString()} detail="recovery or service states" />
              <Metric label="Safety mode" value="READ" detail="descriptor inspection only" accent />
            </div>

            <div className="content-grid">
              <section className="device-list panel">
                <div className="panel-heading"><div><p className="eyebrow">CONNECTED HARDWARE</p><h3>Device inventory</h3></div><span>{devices.length}</span></div>
                {devices.length === 0 ? (
                  <div className="empty-state"><div className="port-icon">⌁</div><h4>No scan results yet</h4><p>Connect a device, then run a descriptor scan.</p></div>
                ) : (
                  <div className="device-rows">
                    {devices.map((device, index) => (
                      <button className={`device-row ${selected === index ? "selected" : ""}`} key={`${device.bus_number}-${device.address}-${device.vendor_id}-${device.product_id}`} onClick={() => setSelected(index)}>
                        <span className="device-orb">{device.platform === "Apple" ? "A" : "U"}</span>
                        <span><strong>{device.product_name || "USB Device"}</strong><small>{hex(device.vendor_id)}:{hex(device.product_id)}</small></span>
                        <b>{device.mode}</b>
                      </button>
                    ))}
                  </div>
                )}
              </section>

              <section className="details panel">
                <div className="panel-heading"><div><p className="eyebrow">SIGNAL REPORT</p><h3>Device details</h3></div></div>
                {active ? (
                  <div className="detail-body">
                    <div className="device-title"><span className="device-orb large">{active.platform === "Apple" ? "A" : "U"}</span><div><h4>{active.product_name || "USB Device"}</h4><p>{active.manufacturer || active.vendor_name || "Unknown manufacturer"}</p></div></div>
                    <dl>
                      <Detail label="Hardware ID" value={`${hex(active.vendor_id)}:${hex(active.product_id)}`} />
                      <Detail label="Mode" value={active.mode} />
                      <Detail label="Platform" value={active.platform} />
                      <Detail label="Transport" value={active.transport} />
                      <Detail label="Bus / Address" value={`${active.bus_number} / ${active.address}`} />
                      <Detail label="Serial" value={active.serial_number || "Not exposed"} />
                    </dl>
                  </div>
                ) : <div className="empty-state compact"><p>Select a detected device to open its signal report.</p></div>}
              </section>
            </div>
          </>
        ) : (
          <>
            <header className="topbar">
              <div><p className="eyebrow">MEDIA BUILDER</p><h1>{host} host → Windows, macOS, and Linux targets.</h1></div>
              <div className="runtime-pill"><i className="online" />Shared planner</div>
            </header>

            <div className="hero-panel">
              <div>
                <span className="status-label">CROSS-PLATFORM MATRIX</span>
                <h2>One BootForge core. Three native host variants.</h2>
                <p>The planner reports what this host can build directly, what needs a native backend, and where Apple-specific installer rules apply.</p>
              </div>
            </div>

            <div className="metric-grid">
              <Metric label="Host" value={host} detail="current desktop platform" />
              <Metric label="Targets" value={matrix.length.toString()} detail="media workflows modeled" />
              <Metric label="Core" value="1" detail="shared Rust capability engine" accent />
            </div>

            <div className="device-rows">
              {matrix.map((cap) => (
                <article className="panel" key={cap.target}>
                  <div className="panel-heading">
                    <div><p className="eyebrow">{levelLabel(cap.level)}</p><h3>{targetLabel(cap.target)}</h3></div>
                    <span>{cap.backend || "planner"}</span>
                  </div>
                  <div className="detail-body">
                    <p>{cap.notes}</p>
                    <div className="recommendation">
                      <span>NEXT ENGINE LAYER</span>
                      <strong>{cap.level === "PreparationOnly" ? "Validate and stage source material" : "Connect host-native write backend"}</strong>
                    </div>
                  </div>
                </article>
              ))}
            </div>
          </>
        )}
      </section>
    </main>
  );
}

function Metric({ label, value, detail, accent = false }: { label: string; value: string; detail: string; accent?: boolean }) {
  return <article className={`metric ${accent ? "accent" : ""}`}><span>{label}</span><strong>{value}</strong><p>{detail}</p></article>;
}

function Detail({ label, value }: { label: string; value: string }) {
  return <div><dt>{label}</dt><dd>{value}</dd></div>;
}

createRoot(document.getElementById("root")!).render(<React.StrictMode><App /></React.StrictMode>);
