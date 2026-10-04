import { Component, signal } from "@angular/core";
import { invoke } from "@tauri-apps/api/core";

@Component({
  selector: "app-root",
  templateUrl: "./app.component.html",
  styleUrl: "./app.component.css",
})
export class AppComponent {
  protected readonly busy = signal(false);
  protected readonly failed = signal(false);

  /** Re-open the video. The app already did this once on launch. */
  protected async again(): Promise<void> {
    this.busy.set(true);
    this.failed.set(false);
    try {
      await invoke("rickroll");
    } catch {
      this.failed.set(true);
    } finally {
      this.busy.set(false);
    }
  }
}
