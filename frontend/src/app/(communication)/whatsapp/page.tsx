"use client";

import { useEffect, useMemo, useRef, useState } from "react";
import type { AxiosInstance } from "axios";
import { toast } from "sonner";
import { useSession } from "next-auth/react";
import { cn } from "@/lib/utils";
import { useAxiosAuth } from "@/lib/axios-auth";
import { useTwilioVoice } from "@/hooks/use-twilio-voice";
import { BACKEND_URL } from "@/lib/constant";
import { Avatar, AvatarFallback } from "@/components/ui/avatar";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { Separator } from "@/components/ui/separator";
import {
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuItem,
  DropdownMenuTrigger,
} from "@/components/ui/dropdown-menu";
import { Skeleton } from "@/components/ui/skeleton";
import {
  MessageCircle,
  Mic,
  MoreVertical,
  Paperclip,
  Phone,
  Plus,
  Search,
  Send,
  Square,
} from "lucide-react";

type TicketPreview = {
  ticket_id: number;
  contact_number: string;
  profile_name: string | null;
  last_message_body: string | null;
  last_message_date: string;
  status: string;
};

type TicketListResponse = {
  tickets: TicketPreview[];
};

type ChatMessageResponse = {
  id: number;
  direction: string;
  body: string | null;
  status: string;
  created_at: string;
  media_url?: string | null;
  media_type?: string | null;
};

type ChatThreadResponse = {
  contact: string;
  messages: ChatMessageResponse[];
};

const AuthMedia = ({
  url,
  type,
  axiosAuth,
}: {
  url: string;
  type: string;
  axiosAuth: AxiosInstance;
}) => {
  const [mediaBlobUrl, setMediaBlobUrl] = useState<string | null>(null);
  const objectUrlRef = useRef<string | null>(null);

  useEffect(() => {
    let active = true;

    const fetchMedia = async () => {
      try {
        const encodedUrl = encodeURIComponent(url);
        const response = await axiosAuth.get(
          `/whatsapp/media?url=${encodedUrl}`,
          {
            responseType: "blob",
          },
        );

        if (active) {
          if (objectUrlRef.current) {
            URL.revokeObjectURL(objectUrlRef.current);
          }
          const objectUrl = URL.createObjectURL(response.data);
          objectUrlRef.current = objectUrl;
          setMediaBlobUrl(objectUrl);
        }
      } catch (error) {
        console.error("Erro ao carregar mídia protegida", error);
      }
    };

    fetchMedia();

    return () => {
      active = false;
      if (objectUrlRef.current) {
        URL.revokeObjectURL(objectUrlRef.current);
        objectUrlRef.current = null;
      }
    };
  }, [url, axiosAuth]);

  if (!mediaBlobUrl) {
    return <Skeleton className="mb-2 h-40 w-full rounded-xl opacity-50" />;
  }

  if (type.startsWith("image/")) {
    return (
      <img
        src={mediaBlobUrl}
        alt="Midia WhatsApp"
        className="mb-2 max-h-60 w-full rounded-xl object-cover"
      />
    );
  }

  if (type.startsWith("audio/")) {
    return (
      <audio controls className="mb-2 w-full">
        <source src={mediaBlobUrl} type={type} />
      </audio>
    );
  }

  return (
    <a
      href={mediaBlobUrl}
      target="_blank"
      rel="noreferrer"
      download="midia_whatsapp"
      className="mb-2 inline-block text-xs underline"
    >
      Baixar arquivo
    </a>
  );
};

const WhatsappPage = () => {
  const axiosAuth = useAxiosAuth();
  const { data, status } = useSession();
  const { ready, connecting, inCall, startCall, hangup } = useTwilioVoice(
    status === "authenticated",
  );

  const [tickets, setTickets] = useState<TicketPreview[]>([]);
  const [loadingTickets, setLoadingTickets] = useState(false);
  const [activeTicket, setActiveTicket] = useState<number | null>(null);
  const [loadingChat, setLoadingChat] = useState(false);
  const [messages, setMessages] = useState<ChatMessageResponse[]>([]);
  const [messageText, setMessageText] = useState("");
  const [search, setSearch] = useState("");
  const [newContactOpen, setNewContactOpen] = useState(false);
  const [newContactNumber, setNewContactNumber] = useState("");
  const [sending, setSending] = useState(false);
  const [page, setPage] = useState(0);
  const [loadingMore, setLoadingMore] = useState(false);
  const [attachment, setAttachment] = useState<File | null>(null);
  const [attachmentPreview, setAttachmentPreview] = useState<string | null>(
    null,
  );
  const [isRecording, setIsRecording] = useState(false);
  const mediaRecorderRef = useRef<MediaRecorder | null>(null);
  const audioChunksRef = useRef<Blob[]>([]);

  const messagesEndRef = useRef<HTMLDivElement>(null);
  const fileInputRef = useRef<HTMLInputElement>(null);
  const objectUrlsRef = useRef<string[]>([]);

  const activeTicketData = useMemo(() => {
    return tickets.find((ticket) => ticket.ticket_id === activeTicket) ?? null;
  }, [tickets, activeTicket]);

  const filteredTickets = useMemo(() => {
    const term = search.trim().toLowerCase();
    if (!term) return tickets;
    return tickets.filter((ticket) => {
      const body = ticket.last_message_body?.toLowerCase() ?? "";
      const name = ticket.profile_name?.toLowerCase() ?? "";
      return (
        ticket.contact_number.includes(term) ||
        body.includes(term) ||
        name.includes(term)
      );
    });
  }, [tickets, search]);

  useEffect(() => {
    const fetchTickets = async () => {
      setLoadingTickets(true);
      try {
        const response =
          await axiosAuth.get<TicketListResponse>("/whatsapp/tickets");

        setTickets(response.data.tickets ?? []);
      } catch {
        toast.error("Erro ao buscar chamados do WhatsApp", {
          position: "top-center",
        });
      } finally {
        setLoadingTickets(false);
      }
    };

    if (status === "authenticated") {
      fetchTickets();
    }
  }, [status, axiosAuth]);

  useEffect(() => {
    const fetchChat = async () => {
      if (!activeTicket) return;
      setLoadingChat(true);
      try {
        const encoded = encodeURIComponent(activeTicket);
        const response = await axiosAuth.get<ChatThreadResponse>(
          `/whatsapp/chat/${encoded}`,
          { params: { page: 0 } },
        );

        setMessages(response.data.messages ?? []);
        setPage(0);
      } catch {
        toast.error("Erro ao buscar conversa", { position: "top-center" });
        setMessages([]);
      } finally {
        setLoadingChat(false);
      }
    };

    if (status === "authenticated") {
      fetchChat();
    }
  }, [status, activeTicket, axiosAuth]);

  const handleLoadMore = async () => {
    if (!activeTicket || loadingMore) return;
    const nextPage = page + 1;
    setLoadingMore(true);
    try {
      const encoded = encodeURIComponent(activeTicket);
      const response = await axiosAuth.get<ChatThreadResponse>(
        `/whatsapp/chat/${encoded}`,
        { params: { page: nextPage } },
      );

      const olderMessages = response.data.messages ?? [];
      if (olderMessages.length > 0) {
        setMessages((prev) => {
          const existingIds = new Set(prev.map((msg) => msg.id));
          const merged = olderMessages.filter(
            (msg) => !existingIds.has(msg.id),
          );
          return [...merged, ...prev];
        });
        setPage(nextPage);
      }
    } catch {
      toast.error("Erro ao carregar mensagens antigas", {
        position: "top-center",
      });
    } finally {
      setLoadingMore(false);
    }
  };

  useEffect(() => {
    messagesEndRef.current?.scrollIntoView({ behavior: "smooth" });
  }, [messages, activeTicket]);

  useEffect(() => {
    return () => {
      objectUrlsRef.current.forEach((url) => URL.revokeObjectURL(url));
      objectUrlsRef.current = [];
    };
  }, []);

  const startRecording = async () => {
    try {
      const stream = await navigator.mediaDevices.getUserMedia({ audio: true });
      const mediaRecorder = new MediaRecorder(stream);
      mediaRecorderRef.current = mediaRecorder;
      audioChunksRef.current = [];

      mediaRecorder.ondataavailable = (event) => {
        if (event.data.size > 0) {
          audioChunksRef.current.push(event.data);
        }
      };

      mediaRecorder.onstop = () => {
        const audioBlob = new Blob(audioChunksRef.current, {
          type: "audio/ogg",
        });

        const audioFile = new File([audioBlob], "gravacao_de_voz.ogg", {
          type: "audio/ogg",
        });

        setAttachment(audioFile);

        const previewUrl = URL.createObjectURL(audioFile);
        objectUrlsRef.current.push(previewUrl);
        setAttachmentPreview(previewUrl);

        stream.getTracks().forEach((track) => track.stop());
      };

      mediaRecorder.start();
      setIsRecording(true);
    } catch {
      toast.error(
        "Permissão de microfone negada ou dispositivo não encontrado.",
      );
    }
  };

  const stopRecording = () => {
    if (mediaRecorderRef.current && isRecording) {
      mediaRecorderRef.current.stop();
      setIsRecording(false);
    }
  };

  const handleStartConversation = () => {
    const trimmed = newContactNumber.trim();
    if (!trimmed) return;

    setNewContactNumber("");
    setNewContactOpen(false);
    setSearch("");

    setTickets((prev) => {
      const existing = prev.find((t) => t.contact_number === trimmed);

      if (existing) {
        setActiveTicket(existing.ticket_id);
        return prev;
      }

      setActiveTicket(0);

      return [
        {
          ticket_id: 0,
          contact_number: trimmed,
          profile_name: null,
          last_message_body: null,
          last_message_date: new Date().toISOString(),
          status: "open",
        },
        ...prev,
      ];
    });
  };

  const handleSendMessage = async () => {
    const trimmedBody = messageText.trim();
    const targetPhone = activeTicketData?.contact_number;

    if (
      (!trimmedBody && !attachment) ||
      !targetPhone ||
      !activeTicket ||
      sending
    )
      return;

    setSending(true);

    try {
      const agentName = data?.user?.name?.split(" ")[0] || "Atendente";

      const finalMessageBody = trimmedBody
        ? `*Atendente ${agentName}*\n${trimmedBody}`
        : `*Atendente ${agentName}*`;

      const formData = new FormData();

      formData.append("to", targetPhone);
      formData.append("ticket_id", String(activeTicket));
      formData.append("sender_name", agentName);
      formData.append("message", finalMessageBody);

      if (attachment) {
        formData.append("file", attachment);
      }

      await axiosAuth.post("/whatsapp/send", formData, {
        headers: { "Content-Type": "multipart/form-data" },
      });

      const now = new Date().toISOString();
      const previewUrl = attachmentPreview ?? null;
      const mediaType = attachment?.type ?? null;
      const messagePreview = trimmedBody || (attachment ? "Midia enviada" : "");
      setMessages((prev) => [
        ...prev,
        {
          id: Date.now(),
          direction: "outbound",
          body: finalMessageBody || null,
          status: "queued",
          created_at: now,
          media_url: previewUrl,
          media_type: mediaType,
        },
      ]);

      setTickets((prev) => {
        const next = prev.map((ticket) => {
          if (ticket.ticket_id !== activeTicket) return ticket;
          return {
            ...ticket,
            last_message_body: messagePreview || null,
            last_message_date: now,
            direction: "outbound",
          };
        });

        const exists = next.some((ticket) => ticket.ticket_id === activeTicket);

        if (!exists) {
          next.unshift({
            ticket_id: activeTicket,
            contact_number: String(activeTicket),
            profile_name: null,
            last_message_body: messagePreview || null,
            last_message_date: now,
            direction: "outbound",
            status: "online",
          });
        }

        return next.sort((a, b) =>
          b.last_message_date.localeCompare(a.last_message_date),
        );
      });

      setMessageText("");
      setAttachment(null);
      setAttachmentPreview(null);
      if (fileInputRef.current) {
        fileInputRef.current.value = "";
      }
    } catch (error) {
      console.log(error);
      toast.error("Erro ao enviar WhatsApp", { position: "top-center" });
    } finally {
      setSending(false);
    }
  };

  const handlePickFile = () => {
    fileInputRef.current?.click();
  };

  const handleFileChange = (event: React.ChangeEvent<HTMLInputElement>) => {
    const file = event.target.files?.[0] ?? null;
    setAttachment(file);

    if (file) {
      const preview = URL.createObjectURL(file);
      objectUrlsRef.current.push(preview);
      setAttachmentPreview(preview);
    } else {
      setAttachmentPreview(null);
    }
  };

  const handleRemoveAttachment = () => {
    setAttachment(null);
    setAttachmentPreview(null);
    if (fileInputRef.current) {
      fileInputRef.current.value = "";
    }
  };

  const handleCallContact = async () => {
    try {
      if (inCall) {
        hangup();
        return;
      }
      if (!activeTicket) return;
      if (!ready) {
        toast.error("Dispositivo de voz nao esta pronto", {
          position: "top-center",
        });
        return;
      }

      await startCall(String(activeTicket));
      toast.success("Ligacao iniciada", { position: "top-center" });
    } catch (error) {
      const message =
        error instanceof Error ? error.message : "Erro ao iniciar ligacao";
      toast.error(message, { position: "top-center" });
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

  const getAvatarFallback = (contact: TicketPreview) => {
    if (contact.profile_name) {
      return contact.profile_name.slice(0, 1).toUpperCase();
    }
    return contact.contact_number.slice(-2);
  };

  const renderMedia = (message: ChatMessageResponse) => {
    if (!message.media_url) return null;

    let type = message.media_type;
    if (!type) {
      if (
        message.media_url.endsWith(".ogg") ||
        message.media_url.endsWith(".webm")
      ) {
        type = "audio/ogg";
      } else {
        type = "image/jpeg";
      }
    }

    if (message.media_url.startsWith("https://api.twilio.com/")) {
      return (
        <AuthMedia url={message.media_url} type={type} axiosAuth={axiosAuth} />
      );
    }

    const resolvedUrl = resolveMediaUrl(message.media_url);

    if (type.startsWith("image/")) {
      return (
        <img
          src={resolvedUrl}
          alt="Midia"
          className="mb-2 max-h-60 w-full rounded-xl object-cover"
        />
      );
    }

    if (type.startsWith("audio/")) {
      return (
        <audio controls className="mb-2 w-full">
          <source src={resolvedUrl} type={type} />
        </audio>
      );
    }

    return (
      <a
        href={resolvedUrl}
        target="_blank"
        rel="noreferrer"
        className="mb-2 inline-block text-xs underline"
      >
        Abrir midia
      </a>
    );
  };

  const resolveMediaUrl = (url: string) => {
    if (url.startsWith("blob:") || url.startsWith("data:")) return url;

    if (url.includes("/uploads/")) {
      const filename = url.split("/").pop();
      return `${BACKEND_URL}/uploads/${filename}`;
    }

    if (url.startsWith("http://") || url.startsWith("https://")) return url;
    if (!BACKEND_URL) return url;
    if (url.startsWith("/")) return `${BACKEND_URL}${url}`;
    return `${BACKEND_URL}/${url}`;
  };

  if (status === "loading") {
    return <p className="p-6 text-muted-foreground">Conectando...</p>;
  }

  return (
    <div className="flex h-svh w-full flex-col bg-emerald-50/70 md:flex-row">
      <aside className="flex w-full flex-col border-b bg-background md:h-full md:w-80 md:border-b-0 md:border-r">
        <div className="flex items-center gap-3 px-4 py-4">
          <div className="flex h-9 w-9 items-center justify-center rounded-xl bg-emerald-100 text-emerald-600">
            <MessageCircle size={18} />
          </div>
          <div>
            <p className="text-sm font-semibold">WhatsApp</p>
            <p className="text-xs text-muted-foreground">
              Conversas e envio de mensagens
            </p>
          </div>
        </div>

        <div className="px-4 pb-4">
          <div className="flex items-center gap-2">
            <div className="relative flex-1">
              <Search
                className="absolute left-3 top-1/2 -translate-y-1/2 text-muted-foreground"
                size={16}
              />
              <Input
                value={search}
                onChange={(event) => setSearch(event.target.value)}
                placeholder="Pesquisar conversa"
                className="h-10 pl-9"
              />
            </div>
            <Button
              type="button"
              size="icon"
              variant="default"
              className="h-10 w-10 rounded-full bg-emerald-500 text-white hover:bg-emerald-500/90"
              onClick={() => setNewContactOpen((prev) => !prev)}
            >
              <Plus size={18} />
            </Button>
          </div>

          {newContactOpen && (
            <div className="mt-3 flex items-center gap-2">
              <Input
                value={newContactNumber}
                onChange={(event) => setNewContactNumber(event.target.value)}
                placeholder="Numero com DDI"
                className="h-9"
              />
              <Button
                type="button"
                size="sm"
                className="bg-emerald-500 text-white hover:bg-emerald-500/90"
                onClick={handleStartConversation}
              >
                Iniciar
              </Button>
            </div>
          )}
        </div>

        <Separator />

        <div className="flex-1 overflow-y-auto">
          {loadingTickets && (
            <div className="space-y-3 px-4 py-4">
              {Array.from({ length: 5 }).map((_, index) => (
                <div key={index} className="flex items-center gap-3">
                  <Skeleton className="h-10 w-10 rounded-full" />
                  <div className="flex-1 space-y-2">
                    <Skeleton className="h-3 w-24" />
                    <Skeleton className="h-3 w-full" />
                  </div>
                </div>
              ))}
            </div>
          )}

          {!loadingTickets && filteredTickets.length === 0 && (
            <div className="px-6 py-10 text-sm text-muted-foreground">
              Nenhuma conversa. Toque em + para iniciar.
            </div>
          )}

          {!loadingTickets && filteredTickets.length > 0 && (
            <div className="flex flex-col">
              {filteredTickets.map((ticket) => {
                const isActive = ticket.contact_number === String(activeTicket);
                const displayName =
                  ticket.profile_name || ticket.contact_number;
                return (
                  <button
                    key={ticket.contact_number}
                    type="button"
                    onClick={() => setActiveTicket(Number(ticket.ticket_id))}
                    className={cn(
                      "flex items-center gap-3 px-4 py-3 text-left transition",
                      isActive ? "bg-emerald-50" : "hover:bg-muted/60",
                    )}
                  >
                    <Avatar className="h-10 w-10">
                      <AvatarFallback>
                        {getAvatarFallback(ticket)}
                      </AvatarFallback>
                    </Avatar>
                    <div className="flex-1 overflow-hidden">
                      <div className="flex items-center justify-between">
                        <p className="text-sm font-semibold">{displayName}</p>
                        <span className="text-xs text-muted-foreground">
                          {formatTime(ticket.last_message_date)}
                        </span>
                      </div>
                      <p className="truncate text-xs text-muted-foreground">
                        {ticket.last_message_body || "Nova conversa"}
                      </p>
                    </div>
                  </button>
                );
              })}
            </div>
          )}
        </div>
      </aside>

      <section className="flex flex-1 flex-col bg-emerald-50/70">
        {activeTicket ? (
          <>
            <div className="flex items-center justify-between border-b bg-background px-5 py-4">
              <div className="flex items-center gap-3">
                <Avatar className="h-10 w-10">
                  <AvatarFallback>
                    {activeTicketData
                      ? getAvatarFallback(activeTicketData)
                      : String(activeTicket).slice(-2)}
                  </AvatarFallback>
                </Avatar>
                <div>
                  <p className="text-sm font-semibold">
                    {activeTicketData?.profile_name ||
                      activeTicketData?.contact_number}
                  </p>
                  <p className="text-xs text-emerald-600">
                    {activeTicketData?.contact_number}
                  </p>
                </div>
              </div>
              <div className="flex items-center gap-2">
                <Button
                  variant="ghost"
                  size="icon"
                  onClick={handleCallContact}
                  disabled={connecting || (!ready && !inCall)}
                >
                  <Phone size={18} />
                </Button>
                <DropdownMenu>
                  <DropdownMenuTrigger asChild>
                    <Button variant="ghost" size="icon">
                      <MoreVertical size={18} />
                    </Button>
                  </DropdownMenuTrigger>
                  <DropdownMenuContent align="end">
                    <DropdownMenuItem>Apagar conversa</DropdownMenuItem>
                  </DropdownMenuContent>
                </DropdownMenu>
              </div>
            </div>

            <div className="flex-1 overflow-y-auto px-6 py-6">
              <div className="min-h-full rounded-3xl bg-emerald-50/80 bg-[radial-gradient(circle,rgba(16,185,129,0.16)_1px,transparent_1px)] bg-size-[22px_22px] p-6">
                {loadingChat && (
                  <div className="space-y-3">
                    {Array.from({ length: 6 }).map((_, index) => (
                      <Skeleton
                        key={index}
                        className={cn(
                          "h-12 w-2/3 rounded-2xl",
                          index % 2 === 0 ? "ml-auto" : "mr-auto",
                        )}
                      />
                    ))}
                  </div>
                )}

                {!loadingChat && messages.length === 0 && (
                  <div className="flex h-full flex-col items-center justify-center text-center text-sm text-muted-foreground">
                    Nenhuma mensagem ainda. Diga ola!
                  </div>
                )}

                {!loadingChat && messages.length > 0 && (
                  <div className="space-y-3">
                    <div className="flex justify-center">
                      <Button
                        type="button"
                        size="sm"
                        variant="ghost"
                        onClick={handleLoadMore}
                        disabled={loadingMore}
                      >
                        {loadingMore ? "Carregando..." : "Ler mais"}
                      </Button>
                    </div>
                    {messages.map((message) => {
                      const outbound = message.direction === "outbound";
                      return (
                        <div
                          key={message.id}
                          className={cn(
                            "flex",
                            outbound ? "justify-end" : "justify-start",
                          )}
                        >
                          <div
                            className={cn(
                              "relative max-w-[70%] rounded-2xl px-4 py-2 shadow-sm",
                              outbound
                                ? "bg-emerald-500 text-white rounded-br-md"
                                : "bg-background text-foreground border rounded-bl-md",
                            )}
                          >
                            {renderMedia(message)}
                            {message.body && (
                              <p className="text-sm whitespace-pre-wrap break-words pr-14">
                                {message.body}
                              </p>
                            )}
                            {!message.body && message.media_url && (
                              <p className="text-xs text-muted-foreground">
                                Midia recebida
                              </p>
                            )}

                            <div
                              className={cn(
                                "mt-1 flex items-center justify-end gap-1 text-[11px]",
                                outbound
                                  ? "text-emerald-100"
                                  : "text-muted-foreground",
                              )}
                            >
                              <span>{formatTime(message.created_at)}</span>

                              {outbound && (
                                <span className="flex items-center text-white text-sm">
                                  {message.status === "sent" && "."}
                                  {message.status === "delivered" && "✓"}
                                  {message.status === "read" && (
                                    <span className="text-white">✓✓</span>
                                  )}
                                </span>
                              )}
                            </div>
                          </div>
                        </div>
                      );
                    })}
                    <div ref={messagesEndRef} />
                  </div>
                )}
              </div>
            </div>

            <div className="border-t bg-background px-6 py-4">
              <input
                ref={fileInputRef}
                type="file"
                accept="image/*,audio/*"
                onChange={handleFileChange}
                className="hidden"
              />
              {attachment && (
                <div className="mb-3 flex items-center justify-between rounded-lg bg-muted px-3 py-2 text-xs">
                  <span className="truncate">{attachment.name}</span>
                  <Button
                    type="button"
                    variant="ghost"
                    size="sm"
                    onClick={handleRemoveAttachment}
                  >
                    Remover
                  </Button>
                </div>
              )}
              <div className="flex items-center gap-3">
                <Button
                  type="button"
                  variant="ghost"
                  size="icon"
                  onClick={handlePickFile}
                  className="h-10 w-10"
                >
                  <Paperclip size={18} />
                </Button>
                {isRecording ? (
                  <Button
                    type="button"
                    variant="destructive"
                    size="icon"
                    onClick={stopRecording}
                    className="h-10 w-10 animate-pulse rounded-full"
                  >
                    <Square size={16} />
                  </Button>
                ) : (
                  <Button
                    type="button"
                    variant="ghost"
                    size="icon"
                    className="h-10 w-10 text-muted-foreground"
                    onClick={startRecording}
                  >
                    <Mic size={20} />
                  </Button>
                )}
                <Input
                  value={messageText}
                  onChange={(event) => setMessageText(event.target.value)}
                  placeholder={
                    isRecording ? "Gravando áudio..." : "Digite uma mensagem"
                  }
                  disabled={isRecording}
                  className="h-12 flex-1"
                  onKeyDown={(event) => {
                    if (event.key === "Enter") {
                      event.preventDefault();
                      handleSendMessage();
                    }
                  }}
                />
                <Button
                  type="button"
                  size="icon"
                  disabled={(!messageText.trim() && !attachment) || sending}
                  onClick={handleSendMessage}
                  className="h-12 w-12 rounded-full bg-emerald-500 text-white hover:bg-emerald-500/90"
                >
                  <Send size={18} />
                </Button>
              </div>
            </div>
          </>
        ) : (
          <div className="flex flex-1 flex-col items-center justify-center text-center">
            <div className="flex h-16 w-16 items-center justify-center rounded-full bg-emerald-100 text-emerald-600">
              <MessageCircle size={24} />
            </div>
            <h2 className="mt-4 text-lg font-semibold">WhatsApp Business</h2>
            <p className="mt-1 text-sm text-muted-foreground">
              Selecione uma conversa a esquerda ou inicie uma nova para comecar
              a enviar mensagens.
            </p>
          </div>
        )}
      </section>
    </div>
  );
};

export default WhatsappPage;
