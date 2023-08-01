// @ts-nocheck

import { Strophe } from "strophe.js";
export class JSConnectionProvider implements ProseConnectionProvider {
  provideConnection(): ProseConnection {
    return new StropheJSConnection();
  }
}

class StropheJSConnection implements ProseConnection {
  private readonly __connection: Strophe.Connection;
  private __eventHandler?: ProseConnectionEventHandler;

  constructor() {
    this.__connection = new Strophe.Connection(
      "wss://chat.prose.org/websocket/",
      { protocol: "wss" }
    );
    this.__connection.rawInput = data => {
      //console.log("RECV", data);
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
        } else if (status === Strophe.Status.CONNECTED) {
          console.log("Strophe is connected.");
          resolve();

          //connection.addHandler(onMessage, null, 'message', null, null,  null);
          //connection.send($pres().tree());
        }
      });
    });
  }

  disconnect() {
    this.__connection.disconnect("logout");
  }

  sendStanza(stanza: string) {
    console.log("Sending stanza", stanza);
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