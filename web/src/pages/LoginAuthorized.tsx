import { createSignal, Match, onMount, Show, Switch } from "solid-js";
import { A, Navigate, useLocation } from "@solidjs/router";
import { CsrfState } from "./Login";
import { fetchUserAuthorization, useAuth } from "../AuthProvider";

export function LoginAuthorized() {
  let [errorMessage, setErrorMessage] = createSignal<string | null>(null);
  const { auth, loading, error, login, returnTo, setReturnTo } = useAuth();

  onMount(() => {
    let csrfStateRaw = window.sessionStorage.getItem("csrfState");
    // window.sessionStorage.removeItem("csrfState");
    if (csrfStateRaw == null) {
      setErrorMessage("No CSRF token found in browser storage");
      return;
    }
    let csrfState: CsrfState = JSON.parse(csrfStateRaw);
    console.debug("loaded csrf state as ", csrfState);
    setReturnTo(csrfState.returnTo);
    /*
        if (Date.now() > csrfState.validUntil) {
            setErrorMessage("Login attempt expired. (You took too long to complete the login)");
            return;
        }
*/
    let query = new URLSearchParams(useLocation().search);
    /*        let otherCsrfToken = query.get("state");
        if (otherCsrfToken == null) {
            setErrorMessage("State parameter not present on request");
            return;
        }

        if (otherCsrfToken !== csrfState.token) {
            setErrorMessage("CSRF tokens do not match");
            return;
        }
*/
    let errorCode = query.get("error");
    if (errorCode === "access_denied") {
      // User pressed cancel, don't show them an error, just return them to where they came from.
      return;
    }
    if (errorCode != null) {
      // Some other error
      let errorMessage = `Authorization completed with error code ${errorCode}`;
      let errorDescription = query.get("error_description");
      if (errorDescription != null) {
        errorMessage += ` (Description: ${errorDescription}`;
      }

      setErrorMessage(errorMessage);
      return;
    }

    let code = query.get("code");
    if (code == null) {
      setErrorMessage("Missing code parameter in query string");
      return;
    }

    login(code);
  });

  let combinedError = () => {
    if (error() != null) {
      return error();
    } else {
      return errorMessage();
    }
  };

  return (
    <Switch>
      <Match when={loading()}>Auth loading...</Match>
      <Match when={combinedError() != null}>
        Login Error :(
        <br />
        Message: {combinedError()}
        <br />
        Feel free to return to where you came from though:{" "}
        <A href={returnTo()}>Click here</A>
      </Match>
      <Match when={true /* else branch */}>
        Would navigate to <code>{returnTo()}</code> (<A href={returnTo()}>Go</A>
        ){/*<Navigate href={returnTo()}/>*/}
      </Match>
    </Switch>
  );
}
