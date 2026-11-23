import { type SubmitEvent, useState } from "react";
import { useTranslation } from "react-i18next";
import { Button } from "@/components/ui/button";
import { Dialog } from "@/components/ui/dialog";
import { Input } from "@/components/ui/input";

type ProtectedConvertDialogProps = {
  open: boolean;
  /** The password the open document was read with; asked when null. */
  knownPassword: string | null;
  onConfirm: (password: string) => void;
  onClose: () => void;
};

// Converting a protected document (specs/08-conversao.md, "Fluxo", step 2):
// warns that the derivative has no password, asks for the password when
// the open document did not need one from the core, and only then converts.
export function ProtectedConvertDialog({
  open,
  knownPassword,
  onConfirm,
  onClose,
}: ProtectedConvertDialogProps) {
  const { t } = useTranslation();
  const [typed, setTyped] = useState("");
  const password = knownPassword ?? typed;
  const submit = (event: SubmitEvent<HTMLFormElement>) => {
    event.preventDefault();
    onConfirm(password);
  };
  return (
    <Dialog
      open={open}
      onOpenChange={(value) => !value && onClose()}
      title={t("conversion.protected.title")}
      closeLabel={t("conversion.protected.close")}
    >
      <form onSubmit={submit} className="flex flex-col gap-3">
        <p className="m-0 text-[13px] text-tx-3">
          {t("conversion.protected.warning")}
        </p>
        {knownPassword === null && (
          <Input
            type="password"
            autoFocus
            aria-label={t("document.password.label")}
            value={typed}
            onChange={(event) => setTyped(event.target.value)}
          />
        )}
        <Button type="submit" className="self-end" disabled={password === ""}>
          {t("conversion.protected.convert")}
        </Button>
      </form>
    </Dialog>
  );
}
