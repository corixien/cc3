import * as NodeModule from "node:module";

NodeModule.register("./electron-stub.mjs", import.meta.url);
