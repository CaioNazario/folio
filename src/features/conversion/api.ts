import { invoke } from "@tauri-apps/api/core";
import { listen, type UnlistenFn } from "@tauri-apps/api/event";
import type { ConversionEvent } from "@/lib/bindings/ConversionEvent";
import type { Format } from "@/lib/bindings/Format";

// Queues a conversion (specs/08-conversao.md, "Execução"); the job id comes
// back, and everything else arrives as `conversion` events.
export function convertDocument(
  id: string,
  target: Format,
  password: string | null,
): Promise<number> {
  return invoke<number>("convert_document", { id, target, password });
}

export function cancelConversion(job: number): Promise<void> {
  return invoke("cancel_conversion", { job });
}

export function onConversionEvent(
  handler: (event: ConversionEvent) => void,
): Promise<UnlistenFn> {
  return listen<ConversionEvent>("conversion", (event) =>
    handler(event.payload),
  );
}
