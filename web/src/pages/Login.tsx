import { onMount } from "solid-js";
import arrayBufferToHex from "array-buffer-to-hex";
import config from "../../config";
import { useSearchParams } from "@solidjs/router";

export interface CsrfState {
  token: string;
  /**
   * Unix timestamp (milliseconds)
   */
  validUntil: number;
  returnTo: string;
}

export function Login() {
  onMount(() => {
    let randomBytes = window.crypto.getRandomValues(new Uint8Array(32)).buffer; // 256 bits of entropy (32 * 8 bits)
    let csrfToken = arrayBufferToHex(randomBytes);

    let [searchParams] = useSearchParams();
    let returnTo = searchParams["returnTo"];
    if (returnTo == null || returnTo == "") {
      returnTo = "/";
    }

    let csrfState: CsrfState = {
      token: csrfToken,
      validUntil: Date.now() + 10 * 60 * 1000, // 10 minutes
      returnTo,
    };
    console.debug("setting csrfState to ", csrfState);
    window.sessionStorage.setItem("csrfState", JSON.stringify(csrfState));

    let url = "https://id.twitch.tv/oauth2/authorize";
    url += `?client_id=${encodeURIComponent(config.twitchApi.clientId)}`;
    url += `&redirect_uri=${encodeURIComponent(config.twitchApi.redirectUri)}`;
    url += `&response_type=code`;
    url += `&scope=`;
    url += `&state=${encodeURIComponent(csrfToken)}`;

    window.location.replace(url);
  });
  return <>Sending you to Twitch...</>;
}
