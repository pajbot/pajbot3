import { A, Route, Routes, useLocation } from "@solidjs/router";
import type { Component } from "solid-js";
import { Login } from "./pages/Login";
import { LoginAuthorized } from "./pages/LoginAuthorized";
import { useAuth, UserAuthorization } from "./AuthProvider";
import { LoginError } from "./pages/LoginError";

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
          <A href={`/login?returnTo=${encodeURIComponent(loginLinkShouldReturnTo())}`}>Login</A>
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
          </a>
        </li>
      </ul>
      returnTo: {JSON.stringify(returnTo())}
      <br />
      auth-auth: {JSON.stringify(auth())}
      <br />
      auth-loading: {String(loading())}
      <br />
      auth-error: {String(error())}
      <br />
      <h1 class="text-3xl font-bold underline">Hello world!</h1>
      <hr />
      <Routes>
        <Route path="/login" component={Login} />
        <Route path="/login/authorized" component={LoginAuthorized} />
        <Route path="/login/error" component={LoginError} />
        <Route path="/" component={() => <>Home sweet home</>} />
        <Route path="*" element={<>404 Not found :(</>} />
      </Routes>
    </div>
  );
};

export default App;
