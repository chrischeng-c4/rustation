import type { Options } from "@wdio/types";
import { spawn, type ChildProcess } from "child_process";
import path from "path";

let tauriDriver: ChildProcess;

export const config: Options.Testrunner = {
  runner: "local",

  autoCompileOpts: {
    autoCompile: true,
    tsNodeOpts: {
      project: "./tsconfig.json",
      transpileOnly: true,
    },
  },

  specs: ["./tests/e2e/**/*.spec.ts"],
  exclude: [],

  maxInstances: 1,

  // Connect to tauri-driver WebDriver server
  hostname: "localhost",
  port: 4444,

  capabilities: [
    {
      // @ts-expect-error custom tauri capability
      "tauri:options": {
        application: path.resolve("./target/debug/rstn"),
      },
      browserName: "wry",  // Tauri's WebView
    },
  ],

  logLevel: "info",
  bail: 0,

  waitforTimeout: 10000,
  connectionRetryTimeout: 120000,
  connectionRetryCount: 3,

  framework: "mocha",
  reporters: ["spec"],

  mochaOpts: {
    ui: "bdd",
    timeout: 60000,
  },

  // Start tauri-driver before tests
  onPrepare: async function () {
    return new Promise<void>((resolve, reject) => {
      tauriDriver = spawn("tauri-driver", [], {
        stdio: ["ignore", "pipe", "pipe"],
      });

      let started = false;

      tauriDriver.stdout?.on("data", (data) => {
        const output = data.toString();
        console.log(`[tauri-driver] ${output}`);
        if (output.includes("listening") && !started) {
          started = true;
          resolve();
        }
      });

      tauriDriver.stderr?.on("data", (data) => {
        console.error(`[tauri-driver] ${data}`);
      });

      tauriDriver.on("error", (err) => {
        reject(err);
      });

      // Fallback timeout
      setTimeout(() => {
        if (!started) {
          resolve();
        }
      }, 3000);
    });
  },

  // Stop tauri-driver after tests
  onComplete: function () {
    tauriDriver?.kill();
  },
};
