import { useCallback, useEffect, useRef, useState } from "react";
import { useTranslation } from "react-i18next";
import { useToast } from "@/components/ui/toast-context";
import type { ConversionEvent } from "@/lib/bindings/ConversionEvent";
import type { Document } from "@/lib/bindings/Document";
import type { Format } from "@/lib/bindings/Format";
import { errorMessage, isBridgeError } from "@/lib/i18n/errors";
import { useFormat } from "@/lib/i18n/useFormat";
import { cancelConversion, convertDocument, onConversionEvent } from "./api";
import { endText, runningText } from "./conversionText";
import {
  applyEvent,
  EMPTY_QUEUE,
  type QueueState,
  withName,
} from "./queueState";

export type ConvertRequest = {
  id: string;
  name: string;
  target: Format;
  password: string | null;
};

// Calls `handler` with every conversion event while mounted.
function useConversionEvents(handler: (event: ConversionEvent) => void) {
  const latest = useRef(handler);
  useEffect(() => {
    latest.current = handler;
  });
  useEffect(() => {
    const unlisten = onConversionEvent((event) => latest.current(event));
    return () => void unlisten.then((stop) => stop());
  }, []);
}

// The queue, kept in a ref so that events arriving in the same tick see
// each other, and mirrored in state to re-render.
function useQueue() {
  const queue = useRef<QueueState>(EMPTY_QUEUE);
  const [snapshot, setSnapshot] = useState<QueueState>(EMPTY_QUEUE);
  const update = useCallback((change: (state: QueueState) => QueueState) => {
    queue.current = change(queue.current);
    setSnapshot(queue.current);
  }, []);
  return { queue, snapshot, update };
}

// While a job runs, a sticky toast shows it, with "Cancelar".
function useRunningToast(snapshot: QueueState) {
  const { t } = useTranslation();
  const format = useFormat();
  const { show } = useToast();
  const text = runningText(snapshot, t, format);
  const job = snapshot.running?.job;
  useEffect(() => {
    if (text === null || job === undefined) return;
    const action = {
      label: t("conversion.cancel"),
      onClick: () => void cancelConversion(job),
    };
    show(text, { sticky: true, action });
  }, [text, job, show, t]);
}

// Turns each event into the queue's next state, the toast that closes a
// job, and the opening of a derivative.
function useEventHandler(
  queue: ReturnType<typeof useQueue>,
  onFinished: (derivative: Document) => void,
) {
  const { t } = useTranslation();
  const { show } = useToast();
  useConversionEvents((event) => {
    const job =
      "job" in event ? queue.queue.current.jobs[event.job] : undefined;
    queue.update((state) => applyEvent(state, event));
    const text = endText(event, job, t);
    if (text !== null) show(text);
    if (event.kind === "finished") onFinished(event.document);
  });
}

// The conversion queue for the UI (specs/08-conversao.md, "Fluxo" and
// "Execução"): asks for conversions and turns their events into toasts.
// `onFinished` opens the derivative.
export function useConversions(onFinished: (derivative: Document) => void) {
  const { show } = useToast();
  const queue = useQueue();
  const { update } = queue;
  useEventHandler(queue, onFinished);
  useRunningToast(queue.snapshot);
  return useCallback(
    ({ id, name, target, password }: ConvertRequest) => {
      update((state) => withName(state, id, name));
      convertDocument(id, target, password).catch((error: unknown) => {
        if (isBridgeError(error)) show(errorMessage(error));
      });
    },
    [update, show],
  );
}
