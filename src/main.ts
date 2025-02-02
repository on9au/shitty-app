// import { invoke } from "@tauri-apps/api/core";
// import { listen } from "@tauri-apps/api/event";
// import { FrontendEvent } from "./bindings/FrontendEvent";

// let greetInputEl: HTMLInputElement | null;
// let greetMsgEl: HTMLElement | null;

// async function funky() {
//   if (greetMsgEl && greetInputEl) {
//     const name = greetInputEl.value;
//     const cringe = await is_name_cringe(name);
//     if (cringe) {
//       greetMsgEl.textContent = "Cringe name detected. Please kill yourself.";
//     } else {
//       greetMsgEl.textContent = await greet(name)
//     }
//   }
// }

// async function greet(name: string): Promise<string> {
//   // if (greetMsgEl && greetInputEl) {
//   // // Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
//   // greetMsgEl.textContent = await invoke("greet", {
//   //   name: greetInputEl.value,
//   // });
//   return await invoke("greet", {
//     name: name,
//   });
//   // }
// }

// async function is_name_cringe(name: string): Promise<boolean> {
//   return await invoke("is_name_cringe", {
//     name: name,
//   });
// }

// window.addEventListener("DOMContentLoaded", () => {
//   greetInputEl = document.querySelector("#greet-input");
//   greetMsgEl = document.querySelector("#greet-msg");
//   document.querySelector("#greet-form")?.addEventListener("submit", (e) => {
//     e.preventDefault();
//     funky();
//   });
// });

// // Communicate to backend that the frontend is ready
// // after DOMContentLoaded
// window.addEventListener("DOMContentLoaded", async () => {
//   await invoke("push_frontend_event", {
//     event: {
//       "type": "FrontendReady"
//     } as FrontendEvent
//   });
// });

// // Backend Event Listener
// // Uses the Tauri `listen` function to listen for the Rust event `backend_event`.
// await listen('backend_event', (event) => {
//   console.log("backend event: " + JSON.stringify(event.payload, null, 2))
// })

import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { FrontendEvent } from "./bindings/FrontendEvent";

window.addEventListener("DOMContentLoaded", () => {
  const eventTypeEl = document.querySelector<HTMLSelectElement>("#event-type");
  const eventDataEl = document.querySelector<HTMLTextAreaElement>("#event-data");
  const sendEventButton = document.querySelector<HTMLButtonElement>("#send-event");
  const backendEventsEl = document.querySelector<HTMLDivElement>("#backend-events");
  const errorMessageEl = document.querySelector<HTMLDivElement>("#error-message");

  sendEventButton?.addEventListener("click", async () => {
    if (eventTypeEl && eventDataEl && errorMessageEl) {
      const eventType = eventTypeEl.value;
      const eventData = eventDataEl.value;

      try {
        const eventObject = JSON.parse(eventData);
        await invoke("push_frontend_event", {
          event: {
            type: eventType,
            ...eventObject
          } as FrontendEvent
        });
        errorMessageEl.textContent = ""; // Clear error message on success
      } catch (error) {
        console.error(error);
        errorMessageEl.textContent = "Invalid JSON input: " + error;
      }
    }
  });

  listen("backend_event", (event) => {
    const eventData = JSON.stringify(event.payload, null, 2);
    const eventEl = document.createElement("div");
    eventEl.textContent = eventData;
    backendEventsEl?.appendChild(eventEl);
  });
});