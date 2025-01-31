import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";

let greetInputEl: HTMLInputElement | null;
let greetMsgEl: HTMLElement | null;

// Backend Event Listener
// Uses the Tauri `listen` function to listen for the Rust event `backend_event`.
await listen('backend_event', (event) => {
  console.log("backend event: " + event)
  let input = event.payload
  console.log("backend event payload: " + input)
})

// Frontend Event Emitter
// Uses the Tauri `invoke` function to call the API function `push_frontend_event`.
await invoke('push_frontend_event', { /* Data */ })

async function funky() {
  if (greetMsgEl && greetInputEl) {
    const name = greetInputEl.value;
    const cringe = await is_name_cringe(name);
    if (cringe) {
      greetMsgEl.textContent = "Cringe name detected. Please kill yourself.";
    } else {
      greetMsgEl.textContent = await greet(name)
    }
  }
}

async function greet(name: string): Promise<string> {
  // if (greetMsgEl && greetInputEl) {
  // // Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
  // greetMsgEl.textContent = await invoke("greet", {
  //   name: greetInputEl.value,
  // });
  return await invoke("greet", {
    name: name,
  });
  // }
}

async function is_name_cringe(name: string): Promise<boolean> {
  return await invoke("is_name_cringe", {
    name: name,
  });
}

window.addEventListener("DOMContentLoaded", () => {
  greetInputEl = document.querySelector("#greet-input");
  greetMsgEl = document.querySelector("#greet-msg");
  document.querySelector("#greet-form")?.addEventListener("submit", (e) => {
    e.preventDefault();
    funky();
  });
});
