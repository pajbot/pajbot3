import { A, Route, Routes, useLocation } from "@solidjs/router";
import type { Component } from "solid-js";
import { Login } from "./pages/Login";
import { LoginAuthorized } from "./pages/LoginAuthorized";
import { useAuth, UserAuthorization } from "./AuthProvider";

const App: Component = () => {
  const { auth, loading, error, logout, setAuth, returnTo } = useAuth();

  let loginLinkShouldReturnTo = () => {
    let pathname = useLocation().pathname;
    if (pathname === "/login/authorized") {
      return returnTo();
    } else {
      return pathname;
    }
  };

  return (
    <div>
      Hi! Epic Layout. These are the pages:
      <ul>
        <li>
          <A href="/">Home</A>
        </li>
        <li>
          <A
            href={`/login?returnTo=${encodeURIComponent(
              loginLinkShouldReturnTo()
            )}`}
          >
            Login
          </A>
        </li>
        <li>
          <a
            href="#"
            onClick={(e) => {
              e.preventDefault();
              setAuth(fauxLogin());
            }}
          >
            Faux Login
          </a>
        </li>
        <li>
          <a
            href="#"
            onClick={(e) => {
              e.preventDefault();
              logout();
            }}
          >
            Logout
          </a>{" "}
          (TODO: invalidate token with API call)
        </li>
      </ul>
      returnTo: {JSON.stringify(returnTo())}
      <br />
      auth-auth: {JSON.stringify(auth())}
      <br />
      auth-loading: {String(loading())}
      <br />
      auth-error: {String(error())}
      <hr />
      <Routes>
        <Route path="/login" component={Login} />
        <Route path="/login/authorized" component={LoginAuthorized} />
        <Route path="/" component={() => <>Home sweet home</>} />
        <Route path="*" element={<>404 Not found :(</>} />
      </Routes>
    </div>
  );
};

function fauxLogin(): UserAuthorization {
  let expiry = new Date(Date.now() + 15 * 1000);
  return {
    access_token:
      "951c8ccd3973ee903b6c3c42a63dbe3c3878c0d6dffdc75b12453019791efc134b6776de5f64221f33bd7ef01c3e43301c3c3c10a056b0f66c8cdf37741ffaf6",
    valid_until: expiry,
    user_details: {
      id: "40286300",
      login: "randers",
      display_name: "randers",
      type: "",
      broadcaster_type: "affiliate",
      description: "",
      profile_image_url:
        "https://static-cdn.jtvnw.net/user-default-pictures-uv/41780b5a-def8-11e9-94d9-784f43822e80-profile_image-300x300.png",
      offline_image_url:
        "https://static-cdn.jtvnw.net/jtv_user_pictures/0622dc8b-c76c-451d-80c1-bea0d915c829-channel_offline_image-1920x1080.png",
      view_count: 1079,
      created_at: new Date("2013-02-13T14:23:31.000Z"),
    },
  };
}

export default App;
