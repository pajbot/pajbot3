import {
  Accessor,
  createContext,
  createMemo,
  createSignal,
  JSX,
  Setter,
  useContext,
} from "solid-js";

export interface UserDetails {
  id: string;
  login: string;
  display_name: string;
  type: string;
  broadcaster_type: string;
  description: string;
  profile_image_url: string;
  offline_image_url: string;
  view_count: number;
  created_at: Date;
}

export interface UserAuthorization {
  access_token: string;
  valid_until: Date;
  user_details: UserDetails;
}

export interface AuthContextData {
  auth: Accessor<UserAuthorization | null>;
  loading: Accessor<boolean>;
  error: Accessor<any | null>;
  setAuth: (auth: UserAuthorization | null) => void;
  login: (code: string) => void;
  logout: () => void;
  returnTo: Accessor<string>;
  setReturnTo: Setter<string>;
}

const AuthContext = createContext<AuthContextData>();

export interface AuthContextProps {
  children: JSX.Element;
}

type LoggedOut = { state: "LoggedOut" };
type LoggedIn = {
  state: "LoggedIn";
  auth: UserAuthorization;
  activeTimeout: number;
};
type LoadingRefresh = {
  state: "LoadingRefresh";
  auth: UserAuthorization;
  abortController: AbortController;
};
type LoadingNew = { state: "LoadingNew"; abortController: AbortController };
type Errored = { state: "Errored"; error: any };
type InternalAuthState =
  | LoggedOut
  | LoggedIn
  | LoadingRefresh
  | LoadingNew
  | Errored;

interface ExternalAuthState {
  auth: UserAuthorization | null;
  loading: boolean;
  error: any | null;
}

export function AuthProvider(props: AuthContextProps) {
  let [returnTo, setReturnTo] = createSignal("/");
  let [internalState, setInternalState] = createSignal<InternalAuthState>({
    state: "LoggedOut",
  });

  let overwriteInternalState = (newState: InternalAuthState) => {
    setInternalState((previousState) => {
      if (previousState.state === "LoggedIn") {
        clearTimeout(previousState.activeTimeout);
      }
      if (
        previousState.state === "LoadingNew" ||
        previousState.state === "LoadingRefresh"
      ) {
        previousState.abortController.abort();
      }
      if ("auth" in newState) {
        setAuthInStorage(newState.auth);
      } else {
        setAuthInStorage(null);
      }
      console.log("new internal auth state", newState);
      return newState;
    });
  };

  let setAuth = (auth: UserAuthorization | null) => {
    if (auth != null) {
      let timeDiff = auth.valid_until.getTime() - Date.now();
      let expired = timeDiff <= 0;

      if (expired) {
        startRefresh(auth);
      } else {
        let activeTimeout = setTimeout(() => startRefresh(auth), timeDiff);
        overwriteInternalState({ state: "LoggedIn", auth, activeTimeout });
      }
    } else {
      overwriteInternalState({ state: "LoggedOut" });
    }
  };

  let startRefresh = (auth: UserAuthorization) => {
    let abortController = new AbortController();
    (async () => {
      try {
        let newAuth = await refreshUserAuthorization(
          auth.access_token,
          abortController.signal
        );
        if (!abortController.signal.aborted) {
          setAuth(newAuth);
        }
      } catch (e) {
        if (!abortController.signal.aborted) {
          console.error("Failed to refresh authorization - Cause: ", e);
          overwriteInternalState({ state: "Errored", error: e });
        }
      }
    })();
    overwriteInternalState({ state: "LoadingRefresh", auth, abortController });
  };

  let login = (code: string) => {
    let abortController = new AbortController();
    (async () => {
      try {
        let newAuth = await fetchUserAuthorization(
          code,
          abortController.signal
        );
        if (!abortController.signal.aborted) {
          setAuth(newAuth);
        }
      } catch (e) {
        if (!abortController.signal.aborted) {
          console.error(
            "Failed to log in (fetch new authorization) - Cause: ",
            e
          );
          overwriteInternalState({ state: "Errored", error: e });
        }
      }
    })();
    overwriteInternalState({ state: "LoadingNew", abortController });
  };

  let logout = () => {
    setAuth(null);
  };

  // init from storage
  setAuth(getAuthFromStorage());

  let externalState = createMemo<ExternalAuthState>(() => {
    let internalStateVal = internalState();
    switch (internalStateVal.state) {
      case "LoggedOut":
        return { auth: null, loading: false, error: null };
      case "LoggedIn":
        return { auth: internalStateVal.auth, loading: false, error: null };
      case "LoadingRefresh":
        return { auth: internalStateVal.auth, loading: true, error: null };
      case "LoadingNew":
        return { auth: null, loading: true, error: null };
      case "Errored":
        return { auth: null, loading: false, error: internalStateVal.error };
    }
  });
  let auth = createMemo(() => externalState().auth);
  let loading = createMemo(() => externalState().loading);
  let error = createMemo(() => externalState().error);

  let contextValue: AuthContextData = {
    auth,
    loading,
    error,
    login,
    logout,
    setAuth,
    returnTo,
    setReturnTo,
  };

  return (
    <AuthContext.Provider value={contextValue}>
      {props.children}
    </AuthContext.Provider>
  );
}

export function useAuth(): AuthContextData {
  let context = useContext(AuthContext);
  if (context == null) {
    throw new Error("Tried to useAuth() without having an <AuthProvider>");
  }
  return context;
}

export async function fetchUserAuthorization(
  code: string,
  abortSignal?: AbortSignal
): Promise<UserAuthorization> {
  console.log("fetch auth! code: " + code);

  const response = await fetch(
    `/api/v1/auth/create?code=${encodeURIComponent(code)}`,
    {
      method: "POST",
      headers: {
        Accept: "application/json",
      },
      signal: abortSignal,
    }
  );

  if (!response.ok) {
    throw Error(response.statusText);
  }
  return deserializeAuth(await response.text());
}

async function refreshUserAuthorization(
  accessToken: string,
  abortSignal?: AbortSignal
): Promise<UserAuthorization> {
  console.log("refresh auth! access token: " + accessToken);

  await new Promise((resolve) => setTimeout(resolve, 20000));

  const response = await fetch(`/api/v1/auth/refresh`, {
    method: "POST",
    headers: {
      Accept: "application/json",
      Authorization: `Bearer ${accessToken}`,
    },
    signal: abortSignal,
  });

  if (!response.ok) {
    throw Error(response.statusText);
  }
  return deserializeAuth(await response.text());
}

function serializeAuth(auth: UserAuthorization | null): string {
  return JSON.stringify(auth);
}

function deserializeAuth(stored: string): any {
  return JSON.parse(stored, (key, value) => {
    if (key === "valid_until" || key === "created_at") {
      return new Date(value);
    }
    return value;
  });
}

const localStorageKey = "auth";
function getAuthFromStorage(): UserAuthorization | null {
  let stored = localStorage.getItem(localStorageKey);
  if (stored != null) {
    return deserializeAuth(stored);
  } else {
    return null;
  }
}

function setAuthInStorage(auth: UserAuthorization | null) {
  if (auth != null) {
    localStorage.setItem(localStorageKey, serializeAuth(auth));
  } else {
    localStorage.removeItem(localStorageKey);
  }
}
