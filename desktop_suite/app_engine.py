"""Boot It Back (BootForge USB) — Read-Only Forensic USB & Bootable Media Observer."""
from __future__ import annotations
from pathlib import Path
from flask import Flask, jsonify, request, send_from_directory
import platform_exclusive

def start_server(port: int, bundle_root: Path, meta: dict):
    app = Flask(__name__)
    ui_dir, assets_dir = bundle_root / "ui", bundle_root / "assets"

    @app.route("/")
    def index(): return send_from_directory(ui_dir, "index.html")
    @app.route("/assets/<path:f>")
    def assets(f): return send_from_directory(assets_dir, f)
    @app.route("/api/health")
    def health(): return jsonify({"ok": True, "app": meta["title"], "mode": "READ_ONLY_FORENSIC"})
    @app.route("/api/platform")
    def plat(): return jsonify(platform_exclusive.get_system_overview())
    @app.route("/api/forensic-scan")
    def fscan():
        return jsonify({
            "usb": platform_exclusive.scan_usb_hardware(),
            "storage": platform_exclusive.scan_physical_disks(),
            "policy": "STRICT_READ_ONLY_OBSERVER",
        })
    @app.route("/api/verify-media", methods=["POST"])
    def vmedia():
        b = request.get_json(force=True, silent=True) or {}
        return jsonify(platform_exclusive.verify_file_evidence(b.get("path", "")))

    app.run(host="127.0.0.1", port=port, debug=False, use_reloader=False)
