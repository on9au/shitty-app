import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { BackendEvent } from "./bindings/BackendEvent";
import { FrontendEvent } from "./bindings/FrontendEvent";

// Type guard for backend events
function isBackendEvent(event: any): event is BackendEvent {
  return 'type' in event;
}

document.addEventListener("DOMContentLoaded", async () => {
  const toggleButton = document.getElementById("toggleSidebar") as HTMLButtonElement;
  const sidebar = document.getElementById("sidebar") as HTMLElement;
  const mainContent = document.getElementById("main-content") as HTMLElement;
  const sidebarLinks = document.querySelectorAll("#sidebar a") as NodeListOf<HTMLAnchorElement>;
  const contentSections = document.querySelectorAll(".content-section") as NodeListOf<HTMLElement>;

  // Set up backend event listener
  await listen('backend-event', (event) => {
    const backendEvent = event.payload;
    if (isBackendEvent(backendEvent)) {
      handleBackendEvent(backendEvent);
    }
  });

  // Function to handle backend events
  function handleBackendEvent(event: BackendEvent) {
    switch (event.type) {
      case 'BackendError':
        showError(event.message);
        break;
      case 'BackendFatal':
        showError(event.message);
        break;
      case 'FatalLostComms':
        showError('Frontend lost communication with backend. Please restart the app and try again.');
        break;
      default:
        console.log('Unhandled backend event:', event);
    }
  }

  // Function to send frontend events
  async function sendFrontendEvent(event: FrontendEvent) {
    try {
      await invoke('push_frontend_event', { event });
    } catch (error) {
      console.error('Error sending frontend event:', error);
      showError('Failed to communicate with backend');
    }
  }

  // UI update functions
  function updateStatus(status: string) {
    const statusElement = document.getElementById('status');
    if (statusElement) {
      statusElement.textContent = status;
    }
  }

  function showError(message: string) {
    const errorElement = document.getElementById('error-message');
    if (errorElement) {
      errorElement.textContent = message;
      errorElement.style.display = 'block';
      setTimeout(() => {
        errorElement.style.display = 'none';
      }, 5000);
    }
  }

  // Sidebar toggle functionality
  if (toggleButton && sidebar && mainContent) {
    toggleButton.addEventListener("click", () => {
      sidebar.classList.toggle("open");
      mainContent.classList.toggle("main-content-shifted");
    });
  }

  // Content switching functionality
  sidebarLinks.forEach(link => {
    link.addEventListener("click", async (e) => {
      e.preventDefault();

      // Hide all content sections
      contentSections.forEach(section => {
        section.classList.remove("active");
      });

      // Show the selected content section
      const sectionId = link.getAttribute("data-section");
      if (sectionId) {
        const targetSection = document.getElementById(sectionId);
        if (targetSection) {
          targetSection.classList.add("active");

          // Send navigation event to backend
          await sendFrontendEvent({
            type: 'Navigation',
            data: sectionId
          });
        }
      }

      // On mobile, close the sidebar after selection
      if (window.innerWidth <= 768) {
        sidebar.classList.remove("open");
        mainContent.classList.remove("main-content-shifted");
      }
    });
  });
});