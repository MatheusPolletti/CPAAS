"use client";

import { useSession, getSession } from "next-auth/react";
import { useEffect } from "react";
import { axiosAuth } from "@/lib/axios";

export const useAxiosAuth = () => {
  const { data: session } = useSession();

  useEffect(() => {
    const requestIntercept = axiosAuth.interceptors.request.use(
      async (config) => {
        let currentSession = session;

        if (
          currentSession?.user?.expiresIn &&
          Date.now() >= currentSession.user.expiresIn - 10000
        ) {
          currentSession = await getSession();
        }

        if (!config.headers["Authorization"] && currentSession?.user?.accessToken) {
          config.headers["Authorization"] = `Bearer ${currentSession.user.accessToken}`;
        }
        
        return config;
      },
      (error) => Promise.reject(error)
    );

    const responseIntercept = axiosAuth.interceptors.response.use(
      (response) => response,
      async (error) => {
        const prevRequest = error?.config;
        
        if (error?.response?.status === 401 && !prevRequest?.sent) {
          prevRequest.sent = true;
          const newSession = await getSession();
          
          if (newSession?.user?.accessToken) {
            prevRequest.headers["Authorization"] = `Bearer ${newSession.user.accessToken}`;
            return axiosAuth(prevRequest);
          }
        }
        
        return Promise.reject(error);
      }
    );

    return () => {
      axiosAuth.interceptors.request.eject(requestIntercept);
      axiosAuth.interceptors.response.eject(responseIntercept);
    };
  }, [session]);

  return axiosAuth;
};