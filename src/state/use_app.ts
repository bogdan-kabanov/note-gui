import { useContext } from "react";
import { AppContext } from "./context";

export function use_app() {
  const value = useContext(AppContext);
  if (!value) {
    throw new Error("Контекст приложения не найден");
  }
  return value;
}
