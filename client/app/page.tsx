"use client";

import { useState } from "react";
import { isConnected, requestAccess } from "@stellar/freighter-api";

export default function MinimalSkillSwap() {
  const [wallet, setWallet] = useState<string>("Not Connected");
  const [skill, setSkill] = useState<string>("");
  const [price, setPrice] = useState<string>("");

  const connectWallet = async () => {
    try {
      if (await isConnected()) {
        const pubKey = await requestAccess();
        setWallet(pubKey);
      }
    } catch (error) {
      setWallet("Connection Failed");
    }
  };

  const handleSwap = async () => {
    if (!skill || !price) return;
    console.log("Submitting:", skill, price);
  };

  return (
    <div style={{ padding: "40px", fontFamily: "monospace" }}>
      <h1>Skill Swap</h1>
      <p>Wallet: {wallet}</p>
      <button onClick={connectWallet} style={{ marginBottom: "20px" }}>
        Connect Freighter
      </button>

      <div>
        <input
          type="text"
          placeholder="Skill"
          value={skill}
          onChange={(e) => setSkill(e.target.value)}
          style={{ display: "block", marginBottom: "10px", padding: "5px" }}
        />
        <input
          type="number"
          placeholder="Price"
          value={price}
          onChange={(e) => setPrice(e.target.value)}
          style={{ display: "block", marginBottom: "10px", padding: "5px" }}
        />
        <button onClick={handleSwap}>Submit to Contract</button>
      </div>
    </div>
  );
}