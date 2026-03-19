import React from "react";
import { Spin } from "antd";

export function PageLoading() {
  return (
    <div style={{ display: "grid", placeItems: "center", height: 240 }}>
      <Spin />
    </div>
  );
}

