"use client";

import { useState, useEffect, useRef, useCallback } from "react";
import { Device, Call } from "@twilio/voice-sdk";
import { useAxiosAuth } from "@/lib/axios-auth";
import { toast } from "sonner";

export const useTwilioVoice = (isAuthenticated: boolean) => {
  const axiosAuth = useAxiosAuth();
  
  const [ready, setReady] = useState(false);
  const [connecting, setConnecting] = useState(false);
  const [inCall, setInCall] = useState(false);
  
  const deviceRef = useRef<Device | null>(null);
  const callRef = useRef<Call | null>(null);

  useEffect(() => {
    let mounted = true;

    const initDevice = async () => {
      if (!isAuthenticated || deviceRef.current) return;

      try {
        const response = await axiosAuth.get<{ token: string }>("/call/token");


        const token = response.data.token;

        const device = new Device(token, {
          codecPreferences: [Call.Codec.Opus, Call.Codec.PCMU],
        });


        device.on("registered", () => {
          if (mounted) setReady(true);
        });

        device.on("error", (error) => {
          if (error.code === 20101) {
            toast.error("Token de voz inválido. Verifique as chaves no Backend.");
          }
        });

        await device.register();
        deviceRef.current = device;

      } catch {
        toast.error("Falha ao conectar com o servidor de voz");
      }
    };

    initDevice();

    return () => {
      mounted = false;
      if (deviceRef.current) {
        deviceRef.current.destroy();
        deviceRef.current = null;
      }
    };
  }, [isAuthenticated, axiosAuth]);

  // Função disparada quando você clica no botão verde
  const startCall = useCallback(async (to: string) => {
    if (!deviceRef.current) throw new Error("Dispositivo não inicializado");

    setConnecting(true);
    try {
      // A MÁGICA ACONTECE AQUI:
      // O `params: { To: to }` é enviado para a Twilio.
      // A Twilio pega esse "To", bate no seu webhook Rust (/call/twiml) 
      // e o Rust usa esse número para gerar o XML <Dial><Number>!
      const call = await deviceRef.current.connect({
        params: { To: to }
      });

      // Eventos da ligação específica
      call.on("accept", () => {
        setConnecting(false);
        setInCall(true); // Muda o botão para vermelho na sua tela
      });

      call.on("disconnect", () => {
        setConnecting(false);
        setInCall(false); // Volta o botão para verde
        callRef.current = null;
      });

      call.on("error", (error) => {
        console.error("Erro na ligação:", error);
        setConnecting(false);
        setInCall(false);
        callRef.current = null;
        toast.error("A ligação caiu ou falhou.");
      });

      callRef.current = call;

    } catch (error) {
      setConnecting(false);
      throw error;
    }
  }, []);

  // Função disparada quando você clica no botão vermelho
  const hangup = useCallback(() => {
    if (deviceRef.current) {
      deviceRef.current.disconnectAll();
    }
  }, []);

  return { ready, connecting, inCall, startCall, hangup };
};