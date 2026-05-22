"use client";

import { useCallback, useEffect, useMemo, useState } from "react";
import { toast } from "sonner";
import { useSession } from "next-auth/react";
import { useAxiosAuth } from "@/lib/axios-auth";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { Separator } from "@/components/ui/separator";
import { Skeleton } from "@/components/ui/skeleton";
import { Delete, Phone, PhoneCall } from "lucide-react";

type CallHistory = {
  id: number;
  call_sid: string;
  from_number: string;
  to_number: string;
  direction: string;
  status: string;
  duration: number | null;
  created_at: string;
};

const dialPad = [
  { value: "1", letters: "" },
  { value: "2", letters: "ABC" },
  { value: "3", letters: "DEF" },
  { value: "4", letters: "GHI" },
  { value: "5", letters: "JKL" },
  { value: "6", letters: "MNO" },
  { value: "7", letters: "PQRS" },
  { value: "8", letters: "TUV" },
  { value: "9", letters: "WXYZ" },
  { value: "*", letters: "" },
  { value: "0", letters: "+" },
  { value: "#", letters: "" },
];

const CallPage = () => {
  const axiosAuth = useAxiosAuth();
  const { status } = useSession();

  const [dialNumber, setDialNumber] = useState("+55");
  const [calling, setCalling] = useState(false);
  const [history, setHistory] = useState<CallHistory[]>([]);
  const [loadingHistory, setLoadingHistory] = useState(false);

  const fetchHistory = useCallback(async () => {
    await Promise.resolve();
    setLoadingHistory(true);
    try {
      const response = await axiosAuth.get<CallHistory[]>("/call/history");
      setHistory(response.data ?? []);
    } catch {
      toast.error("Erro ao buscar historico de chamadas", {
        position: "top-center",
      });
    } finally {
      setLoadingHistory(false);
    }
  }, [axiosAuth]);

  useEffect(() => {
    if (status === "authenticated") {
      fetchHistory();
    }
  }, [status, fetchHistory]);

  const formattedHistory = useMemo(() => {
    return history.map((item) => {
      const contact =
        item.direction === "inbound" ? item.from_number : item.to_number;
      return { ...item, contact };
    });
  }, [history]);

  const handleDial = (value: string) => {
    setDialNumber((prev) => `${prev}${value}`);
  };

  const handleBackspace = () => {
    setDialNumber((prev) => prev.slice(0, -1));
  };

  const handleCall = async () => {
    const trimmed = dialNumber.trim();
    if (!trimmed || trimmed === "+55") {
      toast.error("Informe um numero valido", { position: "top-center" });
      return;
    }

    setCalling(true);
    try {
      await axiosAuth.post("/call/call", { to: trimmed });
      toast.success("Ligacao iniciada", { position: "top-center" });
      setDialNumber("+55");
      fetchHistory();
    } catch {
      toast.error("Erro ao iniciar ligacao", { position: "top-center" });
    } finally {
      setCalling(false);
    }
  };

  const formatTime = (value: string) => {
    const date = new Date(value);
    if (Number.isNaN(date.getTime())) return "";
    return date.toLocaleTimeString("pt-BR", {
      hour: "2-digit",
      minute: "2-digit",
    });
  };

  const formatHistoryDate = (value: string) => {
    const date = new Date(value);
    if (Number.isNaN(date.getTime())) return "";
    const now = new Date();
    const today = now.toDateString();
    const day = date.toDateString();
    if (day === today) {
      return `Hoje, ${formatTime(value)}`;
    }
    const yesterday = new Date(now);
    yesterday.setDate(now.getDate() - 1);
    if (day === yesterday.toDateString()) {
      return "Ontem";
    }
    return date.toLocaleDateString("pt-BR", {
      day: "2-digit",
      month: "short",
    });
  };

  if (status === "loading") {
    return <p className="p-6 text-muted-foreground">Conectando...</p>;
  }

  return (
    <div className="flex h-svh w-full flex-col gap-6 bg-muted/30 p-6">
      <div className="flex items-center gap-3">
        <div className="flex h-10 w-10 items-center justify-center rounded-xl bg-red-100 text-red-600">
          <PhoneCall size={18} />
        </div>
        <div>
          <p className="text-base font-semibold">Fazer ligacao</p>
          <p className="text-sm text-muted-foreground">
            Discador integrado para chamadas de voz.
          </p>
        </div>
      </div>

      <div className="grid flex-1 gap-6 lg:grid-cols-[380px_1fr]">
        <section className="flex flex-col gap-5 rounded-2xl border bg-background p-6 shadow-sm">
          <div className="rounded-2xl bg-slate-900 px-4 py-5 text-center">
            <p className="text-xs uppercase tracking-wide text-white/60">
              Digite o numero
            </p>
            <Input
              value={dialNumber}
              onChange={(event) => setDialNumber(event.target.value)}
              type="tel"
              className="mt-2 h-12 border-0 bg-transparent text-center text-xl font-semibold text-white placeholder:text-white/50 focus-visible:ring-0"
              placeholder="+55"
            />
          </div>

          <div className="grid grid-cols-3 gap-3">
            {dialPad.map((item) => (
              <button
                key={item.value}
                type="button"
                onClick={() => handleDial(item.value)}
                className="flex h-16 flex-col items-center justify-center rounded-2xl bg-muted/50 text-lg font-semibold text-foreground transition hover:bg-muted"
              >
                <span>{item.value}</span>
                {item.letters && (
                  <span className="text-[10px] font-medium text-muted-foreground">
                    {item.letters}
                  </span>
                )}
              </button>
            ))}
          </div>

          <div className="flex items-center justify-center gap-4 pt-2">
            <Button
              type="button"
              onClick={handleCall}
              disabled={calling}
              className="h-16 w-16 rounded-full bg-green-500 text-white shadow-md hover:bg-green-500/90 cursor-pointer"
            >
              <Phone size={20} />
            </Button>
            <Button
              type="button"
              variant="ghost"
              size="icon"
              onClick={handleBackspace}
              className="h-10 w-10 text-muted-foreground"
            >
              <Delete size={18} />
            </Button>
          </div>
        </section>

        <section className="flex flex-col rounded-2xl border bg-background p-6 shadow-sm">
          <div className="flex items-center justify-between">
            <p className="text-sm font-semibold">Recentes</p>
          </div>
          <Separator className="my-4" />

          {loadingHistory && (
            <div className="space-y-3">
              {Array.from({ length: 5 }).map((_, index) => (
                <div key={index} className="flex items-center gap-3">
                  <Skeleton className="h-10 w-10 rounded-full" />
                  <div className="flex-1 space-y-2">
                    <Skeleton className="h-3 w-32" />
                    <Skeleton className="h-3 w-24" />
                  </div>
                  <Skeleton className="h-3 w-16" />
                </div>
              ))}
            </div>
          )}

          {!loadingHistory && formattedHistory.length === 0 && (
            <p className="text-sm text-muted-foreground">
              Nenhuma chamada recente.
            </p>
          )}

          {!loadingHistory && formattedHistory.length > 0 && (
            <div className="flex flex-col">
              {formattedHistory.map((item) => {
                const label =
                  item.direction === "inbound" ? "Recebida" : "Realizada";
                return (
                  <div key={item.id} className="py-3">
                    <div className="flex items-center justify-between">
                      <div>
                        <p className="text-sm font-semibold">{item.contact}</p>
                        <p className="text-xs text-muted-foreground">
                          {label} · {item.status}
                        </p>
                      </div>
                      <span className="text-xs text-muted-foreground">
                        {formatHistoryDate(item.created_at)}
                      </span>
                    </div>
                    <Separator className="mt-3" />
                  </div>
                );
              })}
            </div>
          )}
        </section>
      </div>
    </div>
  );
};

export default CallPage;
