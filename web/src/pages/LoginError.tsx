import { A } from "@solidjs/router";
import { useAuth } from "../AuthProvider";

export function LoginError() {
  let { returnTo, error } = useAuth();

  return (
    <>
      Login Error :(
      <br />
      Message: {error()}
      <br />
      Feel free to return to where you came from though: <A href={returnTo()}>Click here</A>
    </>
  );
}
