/* @refresh reload */
import { render } from "solid-js/web";
import { Router } from "@solidjs/router";

import "./index.css";
import App from "./App";
import { AuthProvider } from "./AuthProvider";

render(
  () => (
    <Router>
      <AuthProvider>
        <App />
      </AuthProvider>
    </Router>
  ),
  document.getElementById("root") as HTMLElement
);
