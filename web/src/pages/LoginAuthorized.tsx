import { createEffect, createSignal, Match, onMount, Show, Switch } from "solid-js";
import { A, Navigate, useNavigate, useSearchParams } from "@solidjs/router";
import { CsrfState } from "./Login";
import { fetchUserAuthorization, useAuth } from "../AuthProvider";

export function LoginAuthorized() {
  const { loading, error, login, setError, returnTo, setReturnTo } = useAuth();

  onMount(() => {
    let [searchParams, setSearchParams] = useSearchParams();
    let otherCsrfToken = searchParams.state;
    let errorCode = searchParams.error;
    let errorDescription = searchParams.error_description;
    let code = searchParams.code;
    // Clear the parameters out of the location bar. replace: true will also prevent these details from staying
    // around in the browser's history - the history entry will be overwritten/replaced
    setSearchParams(
      {
        state: null,
        error: null,
        error_description: null,
        code: null,
        scope: null,
      },
      { replace: true },
    );

    let csrfStateRaw = window.sessionStorage.getItem("csrfState");
    window.sessionStorage.removeItem("csrfState");
    if (csrfStateRaw == null) {
      setError("No CSRF token found in browser storage");
      return;
    }
    let csrfState: CsrfState = JSON.parse(csrfStateRaw);
    console.debug("loaded csrf state as ", csrfState);
    setReturnTo(csrfState.returnTo);

    if (Date.now() > csrfState.validUntil) {
      setError("Login attempt expired. (You took too long to complete the login)");
      return;
    }

    if (otherCsrfToken == null) {
      setError("State parameter not present on request");
      return;
    }

    if (otherCsrfToken !== csrfState.token) {
      setError("CSRF tokens do not match");
      return;
    }

    if (errorCode === "access_denied") {
      // User pressed cancel, don't show them an error, just return them to where they came from.
      return;
    }
    if (errorCode != null) {
      // Some other error
      let errorMessage = `Authorization completed with error code ${errorCode}`;
      if (errorDescription != null) {
        errorMessage += ` (Description: ${errorDescription}`;
      }

      setError(errorMessage);
      return;
    }

    if (code == null) {
      setError("Missing code parameter in query string");
      return;
    }

    login(code);
  });

  const navigate = useNavigate();
  createEffect(() => {
    if (error() != null) {
      navigate("/login/error", { replace: true });
    } else if (!loading()) {
      // success
      navigate(returnTo(), { replace: true });
    }
  });

  return <Show when={loading()}>Please wait...</Show>;
}
