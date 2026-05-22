import axios from "axios";
import { BACKEND_URL } from "./constant";

const BASE_URL = BACKEND_URL;

export default axios.create({
  baseURL: BASE_URL,
  headers: {
    "Content-Type": "application/json",
  },
});

export const axiosAuth = axios.create({
  baseURL: BASE_URL,
  headers: {
    "Content-Type": "application/json",
  },
});