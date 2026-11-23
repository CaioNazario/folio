import type { TFunction } from "i18next";
import type { ConversionEvent } from "@/lib/bindings/ConversionEvent";
import { errorMessage } from "@/lib/i18n/errors";
import type { Job, QueueState } from "./queueState";

// The toast texts of the conversion queue (specs/08-conversao.md, "Execução").

type Format = { percent: (ratio: number) => string };

/** "Convertendo Aula.txt para Markdown… 40%". */
function jobText(job: Job, fraction: number | null, t: TFunction, format: Format): string {
  const values = { name: job.name, format: t(`document.formats.${job.target}`) }
  if (fraction === null) return t('conversion.running', values)
  return t('conversion.runningPercent', { ...values, percent: format.percent(fraction) })
}

/** The running job, with the count of jobs waiting: "… 40% · 1 na fila". */
export function runningText(state: QueueState, t: TFunction, format: Format): string | null {
  const running = state.running
  const job = running && state.jobs[running.job]
  if (!running || !job) return null
  const text = jobText(job, running.fraction, t, format)
  const waiting = state.waiting.length
  return waiting === 0 ? text : t('conversion.waiting', { running: text, count: waiting })
}

/** The toast that closes a job; null for events that do not. */
export function endText(
  event: ConversionEvent,
  job: Job | undefined,
  t: TFunction,
): string | null {
  if (event.kind === "finished")
    return t("conversion.done", {
      derivative: event.document.name,
      original: job?.name ?? "",
    });
  if (event.kind === "failed") return errorMessage(event.error);
  if (event.kind === "cancelled") return t("errors.ConversionCancelled");
  return null;
}
