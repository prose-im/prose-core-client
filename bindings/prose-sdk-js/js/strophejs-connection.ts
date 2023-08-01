// @ts-nocheck

import { Strophe } from "strophe.js";

export class JSConnectionConfig {
  logReceivedStanzas: boolean
  logSentStanzas: boolean

  setLogReceivedStanzas(flag: boolean) {
    this.logReceivedStanzas = flag;
  }

  setLogSentStanzas(flag: boolean): void {
    this.logSentStanzas = flag;
  }
}
export class JSConnectionProvider implements ProseConnectionProvider {
  private readonly __config: JSConnectionConfig

  constructor(config: JSConnectionConfig) {
    this.__config = config;
  }

  provideConnection(): ProseConnection {
    return new StropheJSConnection(this.__config);
  }
}

class StropheJSConnection implements ProseConnection {
  private readonly __config: JSConnectionConfig
  private readonly __connection: Strophe.Connection;
  private __eventHandler?: ProseConnectionEventHandler;

  constructor(config: JSConnectionConfig) {
    this.__config = config;
    this.__connection = new Strophe.Connection(
      "wss://chat.prose.org/websocket/",
      { protocol: "wss" }
    );
    this.__connection.maxRetries = 0;
    this.__connection.rawInput = data => {
      if (this.__config.logReceivedStanzas) {
        console.info("(in)", data);
      }
      if (this.__eventHandler) {
        this.__eventHandler.handleStanza(data);
      }
    };
  }

  async connect(jid: string, password: string) {
    return new Promise<void>((resolve, reject) => {
      this.__connection.connect(jid, password, status => {
        if (status === Strophe.Status.CONNECTING) {
          console.log("Strophe is connecting.");
        } else if (status === Strophe.Status.CONNFAIL) {
          console.log("Strophe failed to connect.");
          reject(new Error("Something went wrong."));
        } else if (status === Strophe.Status.DISCONNECTING) {
          console.log("Strophe is disconnecting.");
        } else if (status === Strophe.Status.DISCONNECTED) {
          console.log("Strophe is disconnected.");
          setTimeout(() => this.__eventHandler.handleDisconnect(null));
        } else if (status === Strophe.Status.CONNECTED) {
          console.log("Strophe is connected.");
          resolve();
        }
      });
    });
  }

  disconnect() {
    this.__connection.disconnect("logout");
  }

  sendStanza(stanza: string) {
    if (this.__config.logSentStanzas) {
      console.info("(out)", stanza);
    }

    const element = new DOMParser().parseFromString(
      stanza,
      "text/xml"
    ).firstElementChild;

    if (!element) {
      throw new Error("Failed to parse stanza");
    }

    this.__connection.send(element);
  }

  setEventHandler(handler: ProseConnectionEventHandler) {
    this.__eventHandler = handler;
  }
}