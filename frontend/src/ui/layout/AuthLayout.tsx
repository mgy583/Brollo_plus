import React from "react";
import { Outlet } from "react-router-dom";

export function AuthLayout() {
  return (
    <div
      style={{
        minHeight: "100%",
        display: "grid",
        placeItems: "center",
        background: "#f7f7f5",
        backgroundImage:
          "repeating-linear-gradient(0deg, transparent, transparent 27px, rgba(0,0,0,0.05) 27px, rgba(0,0,0,0.05) 28px)," +
          "repeating-linear-gradient(90deg, transparent, transparent 27px, rgba(0,0,0,0.03) 27px, rgba(0,0,0,0.03) 28px)",
        padding: 16,
      }}
    >
      <div style={{ position: "relative", width: "100%", maxWidth: 440 }}>
        {/* 装饰圆圈 */}
        <div style={{ position: "absolute", top: -36, left: -24, width: 72, height: 72, border: "2.5px solid rgba(0,0,0,0.1)", borderRadius: "60% 40% 55% 45% / 45% 55% 40% 60%", transform: "rotate(-10deg)", pointerEvents: "none" }} />
        <div style={{ position: "absolute", bottom: -24, right: -16, width: 52, height: 52, border: "2px solid rgba(0,0,0,0.08)", borderRadius: "45% 55% 60% 40% / 55% 45% 55% 45%", transform: "rotate(6deg)", pointerEvents: "none" }} />
        <div
          style={{
            background: "#ffffff",
            border: "2.5px solid #1a1a1a",
            borderRadius: 2,
            padding: "32px 32px 28px",
            boxShadow: "6px 6px 0 #1a1a1a",
            position: "relative",
          }}
        >
          {/* 折角 */}
          <div style={{ position: "absolute", top: 0, right: 0, width: 0, height: 0, borderStyle: "solid", borderWidth: "0 28px 28px 0", borderColor: "transparent #1a1a1a transparent transparent" }} />
          {/* Logo */}
          <div style={{ textAlign: "center", marginBottom: 28 }}>
            <div style={{ fontSize: 32, fontWeight: 700, color: "#1a1a1a", letterSpacing: 2, lineHeight: 1 }}>
              ✏️ Brollo+
            </div>
            <div style={{ fontSize: 13, color: "#888", marginTop: 6, borderTop: "1.5px dashed rgba(0,0,0,0.15)", paddingTop: 6 }}>
              你的专属手账记账本
            </div>
          </div>
          <Outlet />
        </div>
      </div>
    </div>
  );
}
