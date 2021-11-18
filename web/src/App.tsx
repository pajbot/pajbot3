import React from "react";
import "./App.css";
import { BrowserRouter, Link, Outlet, Route, Routes } from "react-router-dom";
import Page1 from "./routes/page1";
import Page2 from "./routes/page2";

function App() {
  let app = (
    <div>
      <h1>pajbot web test</h1>
      <nav>
        <Link to="/page1">test page 1</Link> |{" "}
        <Link to="/page2">test page 2</Link>
      </nav>
      <Outlet />
    </div>
  );

  return (
    <BrowserRouter>
      <Routes>
        <Route path="/" element={app}>
          <Route path="page1" element={<Page1 />} />
          <Route path="page2" element={<Page2 />} />
        </Route>
      </Routes>
    </BrowserRouter>
  );
}

export default App;
