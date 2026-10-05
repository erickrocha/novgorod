import { useEffect, useId, useRef, useState } from "react";
import { useTranslation } from "react-i18next";
import Checkbox from "@/components/form/input/Checkbox";
import Button from "@/components/ui/button/Button";
import { CheckLineIcon } from "@/icons";
import { tenantService } from "@/services/tenantService";
import { useAppSelector } from "@/store/hooks";
import { ROLES } from "@/utils/enums";

export default function TenantListing({ tenantId }: { tenantId: number }) {
  const { user } = useAppSelector((state) => state.auth);
  const ownId = user?.tenantId ?? user?.tenant_id;
  const allowed =
    user?.role === ROLES.SYS_ADMIN ||
    (user?.role === ROLES.TENANT_OWNER && ownId === tenantId);

  return allowed ? <ListingControl key={tenantId} tenantId={tenantId} /> : null;
}

function ListingControl({ tenantId }: { tenantId: number }) {
  const { t } = useTranslation();
  const headingId = useId();
  const [confirmed, setConfirmed] = useState<boolean | null>(null);
  const [selected, setSelected] = useState(false);
  const [loading, setLoading] = useState(true);
  const [readError, setReadError] = useState(false);
  const [saving, setSaving] = useState(false);
  const [feedback, setFeedback] = useState<"saved" | "error" | null>(null);
  const [attempt, setAttempt] = useState(0);
  const savePending = useRef(false);

  useEffect(() => {
    let active = true;
    tenantService
      .getListing(tenantId)
      .then((response) => {
        if (active) {
          setConfirmed(response.listed);
          setSelected(response.listed);
        }
      })
      .catch(() => {
        if (active) setReadError(true);
      })
      .finally(() => {
        if (active) setLoading(false);
      });
    return () => {
      active = false;
    };
  }, [tenantId, attempt]);

  const retry = () => {
    setLoading(true);
    setReadError(false);
    setAttempt((value) => value + 1);
  };

  const save = async () => {
    if (confirmed === null || selected === confirmed || savePending.current)
      return;
    savePending.current = true;
    setSaving(true);
    setFeedback(null);
    try {
      const response = await tenantService.setListing(tenantId, selected);
      setConfirmed(response.listed);
      setSelected(response.listed);
      setFeedback("saved");
    } catch {
      setFeedback("error");
    } finally {
      savePending.current = false;
      setSaving(false);
    }
  };

  return (
    <section
      aria-labelledby={headingId}
      className="space-y-3 border-gray-200 pb-5 dark:border-gray-800 border-b"
    >
      <h3
        id={headingId}
        className="text-sm font-semibold text-gray-900 dark:text-gray-100"
      >
        {t("tenants.listing.title")}
      </h3>
      <div className="gap-4 flex flex-wrap items-center">
        <Checkbox
          label={t("tenants.listing.label")}
          checked={selected}
          disabled={loading || saving || confirmed === null}
          onChange={(value) => {
            setSelected(value);
            setFeedback(null);
          }}
        />
        <Button
          size="sm"
          startIcon={<CheckLineIcon aria-hidden="true" className="size-4" />}
          onClick={save}
          disabled={
            loading || saving || confirmed === null || selected === confirmed
          }
        >
          {t(saving ? "common.saving" : "tenants.listing.save")}
        </Button>
      </div>
      {loading && (
        <p role="status" className="text-sm text-gray-600 dark:text-gray-400">
          {t("common.loading")}
        </p>
      )}
      {readError && (
        <div className="gap-3 flex flex-wrap items-center">
          <p
            role="alert"
            className="text-sm text-error-600 dark:text-error-400"
          >
            {t("tenants.listing.readError")}
          </p>
          <Button size="sm" variant="outline" onClick={retry}>
            {t("tenants.listing.retry")}
          </Button>
        </div>
      )}
      {feedback === "error" && (
        <p role="alert" className="text-sm text-error-600 dark:text-error-400">
          {t("tenants.listing.saveError")}
        </p>
      )}
      {feedback === "saved" && (
        <p
          role="status"
          className="text-sm text-success-600 dark:text-success-400"
        >
          {t("tenants.listing.saved")}
        </p>
      )}
    </section>
  );
}
