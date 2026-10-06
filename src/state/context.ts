import { createContext } from "react";
import type { app_api } from "./app_context";

export const AppContext = createContext<app_api | null>(null);
