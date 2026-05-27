"use client";

import { useEffect, useMemo, useRef, useState } from "react";
import { toast } from "sonner";
import { useSession } from "next-auth/react";
import { cn } from "@/lib/utils";
import { useAxiosAuth } from "@/lib/axios-auth";
import { useTwilioVoice } from "@/hooks/use-twilio-voice";
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
  MessageSquare,
  MoreVertical,
  Phone,
  Plus,
  Search,
  Send,
} from "lucide-react";

type ContactPreview = {
  contact_number: string;
  name: string | null;
  last_message_body: string | null;
  last_message_date: string;
  direction: string;
};

type ContactListResponse = {
  contacts: ContactPreview[];
};

type ChatMessageResponse = {
  id: number;
  direction: string;
  body: string | null;
  status: string | null;
  created_at: string;
};

type ChatThreadResponse = {
  contact: string;
  messages: ChatMessageResponse[];
};

const SmsPage = () => {
  const axiosAuth = useAxiosAuth();
  const { status } = useSession();
  const { ready, connecting, inCall, startCall, hangup } = useTwilioVoice(
    status === "authenticated",
  );

  const [contacts, setContacts] = useState<ContactPreview[]>([]);
  const [loadingContacts, setLoadingContacts] = useState(false);
  const [activeContact, setActiveContact] = useState<string | null>(null);
  const [loadingChat, setLoadingChat] = useState(false);
  const [messages, setMessages] = useState<ChatMessageResponse[]>([]);
  const [messageText, setMessageText] = useState("");
  const [search, setSearch] = useState("");
  const [newContactOpen, setNewContactOpen] = useState(false);
  const [newContactNumber, setNewContactNumber] = useState("");
  const [sending, setSending] = useState(false);
  const [savingContact, setSavingContact] = useState(false);
  const [contactName, setContactName] = useState("");

  const messagesEndRef = useRef<HTMLDivElement>(null);

  const activeContactData = useMemo(() => {
    return (
      contacts.find((contact) => contact.contact_number === activeContact) ??
      null
    );
  }, [contacts, activeContact]);

  const filteredContacts = useMemo(() => {
    const term = search.trim().toLowerCase();
    if (!term) return contacts;
    return contacts.filter((contact) => {
      const body = contact.last_message_body?.toLowerCase() ?? "";
      const name = contact.name?.toLowerCase() ?? "";
      return (
        contact.contact_number.toLowerCase().includes(term) ||
        body.includes(term) ||
        name.includes(term)
      );
    });
  }, [contacts, search]);

  useEffect(() => {
    const fetchContacts = async () => {
      setLoadingContacts(true);
      try {
        const response =
          await axiosAuth.get<ContactListResponse>("/sms/contacts");

        setContacts(response.data.contacts ?? []);
      } catch {
        toast.error("Erro ao buscar contatos SMS", { position: "top-center" });
      } finally {
        setLoadingContacts(false);
      }
    };

    if (status === "authenticated") {
      fetchContacts();
    }
  }, [status, axiosAuth]);

  useEffect(() => {
    const fetchChat = async () => {
      if (!activeContact) return;
      setLoadingChat(true);
      try {
        const encoded = encodeURIComponent(activeContact);
        const response = await axiosAuth.get<ChatThreadResponse>(
          `/sms/chat/${encoded}`,
          { params: { page: 0 } },
        );
        setMessages(response.data.messages ?? []);
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
  }, [status, activeContact, axiosAuth]);

  useEffect(() => {
    messagesEndRef.current?.scrollIntoView({ behavior: "smooth" });
  }, [messages, activeContact]);

  const handleStartConversation = () => {
    const trimmed = newContactNumber.trim();
    if (!trimmed) return;

    setActiveContact(trimmed);
    setNewContactNumber("");
    setNewContactOpen(false);
    setContactName("");
    setSearch("");

    setContacts((prev) => {
      const existing = prev.find(
        (contact) => contact.contact_number === trimmed,
      );
      if (existing) return prev;
      return [
        {
          contact_number: trimmed,
          name: null,
          last_message_body: null,
          last_message_date: new Date().toISOString(),
          direction: "outbound",
        },
        ...prev,
      ];
    });
  };

  const handleSendMessage = async () => {
    const trimmedBody = messageText.trim();
    if (!trimmedBody || !activeContact || sending) return;

    setSending(true);
    try {
      await axiosAuth.post("/sms/send", {
        to: activeContact,
        body: trimmedBody,
      });

      const now = new Date().toISOString();
      setMessages((prev) => [
        ...prev,
        {
          id: Date.now(),
          direction: "outbound",
          body: trimmedBody,
          status: "sent",
          created_at: now,
        },
      ]);

      setContacts((prev) => {
        const next = prev.map((contact) => {
          if (contact.contact_number !== activeContact) return contact;
          return {
            ...contact,
            last_message_body: trimmedBody,
            last_message_date: now,
            direction: "outbound",
          };
        });

        const exists = next.some(
          (contact) => contact.contact_number === activeContact,
        );
        if (!exists) {
          next.unshift({
            contact_number: activeContact,
            name: null,
            last_message_body: trimmedBody,
            last_message_date: now,
            direction: "outbound",
          });
        }

        return next.sort((a, b) =>
          b.last_message_date.localeCompare(a.last_message_date),
        );
      });

      setMessageText("");
    } catch {
      toast.error("Erro ao enviar SMS", { position: "top-center" });
    } finally {
      setSending(false);
    }
  };

  const handleSaveContact = async () => {
    const trimmedName = contactName.trim();
    if (!activeContact || !trimmedName || savingContact) return;
    setSavingContact(true);
    try {
      await axiosAuth.post("/sms/contact/save", {
        phone_number: activeContact,
        name: trimmedName,
      });

      setContacts((prev) =>
        prev.map((contact) =>
          contact.contact_number === activeContact
            ? { ...contact, name: trimmedName }
            : contact,
        ),
      );

      toast.success("Contato salvo com sucesso", {
        position: "top-center",
      });
    } catch {
      toast.error("Erro ao salvar contato", { position: "top-center" });
    } finally {
      setSavingContact(false);
    }
  };

  const handleCallContact = async () => {
    try {
      if (inCall) {
        hangup();
        return;
      }
      if (!activeContact) return;
      if (!ready) {
        toast.error("Dispositivo de voz nao esta pronto", {
          position: "top-center",
        });
        return;
      }

      await startCall(activeContact);
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

  if (status === "loading") {
    return <p className="p-6 text-muted-foreground">Conectando...</p>;
  }

  return (
    <div className="flex h-svh w-full flex-col bg-muted/40 md:flex-row">
      <aside className="flex w-full flex-col border-b bg-background md:h-full md:w-80 md:border-b-0 md:border-r">
        <div className="flex items-center gap-3 px-4 py-4">
          <div className="flex h-9 w-9 items-center justify-center rounded-xl bg-blue-50 text-blue-600">
            <MessageSquare size={18} />
          </div>
          <div>
            <p className="text-sm font-semibold">SMS</p>
            <p className="text-xs text-muted-foreground">
              Historico de contatos e envio de mensagens
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
                placeholder="Pesquisar contato"
                className="h-10 pl-9"
              />
            </div>
            <Button
              type="button"
              size="icon"
              variant="default"
              className="h-10 w-10 rounded-full"
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
              <Button type="button" size="sm" onClick={handleStartConversation}>
                Iniciar
              </Button>
            </div>
          )}
        </div>

        <Separator />

        <div className="flex-1 overflow-y-auto">
          {loadingContacts && (
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

          {!loadingContacts && filteredContacts.length === 0 && (
            <div className="px-6 py-10 text-sm text-muted-foreground">
              Nenhum contato. Toque em + para iniciar.
            </div>
          )}

          {!loadingContacts && filteredContacts.length > 0 && (
            <div className="flex flex-col">
              {filteredContacts.map((contact) => {
                const isActive = contact.contact_number === activeContact;
                return (
                  <button
                    key={contact.contact_number}
                    type="button"
                    onClick={() => {
                      setActiveContact(contact.contact_number);
                      setContactName(contact.name ?? "");
                    }}
                    className={cn(
                      "flex items-center gap-3 px-4 py-3 text-left transition",
                      isActive ? "bg-blue-50" : "hover:bg-muted/60",
                    )}
                  >
                    <Avatar className="h-10 w-10">
                      <AvatarFallback>
                        {contact.contact_number.slice(-2)}
                      </AvatarFallback>
                    </Avatar>
                    <div className="flex-1 overflow-hidden">
                      <div className="flex items-center justify-between">
                        <p className="text-sm font-semibold">
                          {contact.name || contact.contact_number}
                        </p>
                        <span className="text-xs text-muted-foreground">
                          {formatTime(contact.last_message_date)}
                        </span>
                      </div>
                      <p className="truncate text-xs text-muted-foreground">
                        {contact.last_message_body || "Nova conversa"}
                      </p>
                    </div>
                  </button>
                );
              })}
            </div>
          )}
        </div>
      </aside>

      <section className="flex flex-1 flex-col bg-muted/40">
        {activeContact ? (
          <>
            <div className="flex items-center justify-between border-b bg-background px-5 py-4">
              <div className="flex items-center gap-3">
                <Avatar className="h-10 w-10">
                  <AvatarFallback>{activeContact.slice(-2)}</AvatarFallback>
                </Avatar>
                <div>
                  <p className="text-sm font-semibold">
                    {activeContactData?.name || activeContact}
                  </p>
                  <p className="text-xs text-muted-foreground">
                    {activeContactData?.last_message_body
                      ? "Conversa ativa"
                      : "Nova conversa"}
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

            <div className="border-b bg-background px-5 py-3">
              <div className="flex flex-wrap items-center gap-3">
                <Input
                  value={contactName}
                  onChange={(event) => setContactName(event.target.value)}
                  placeholder="Salvar nome do contato"
                  className="h-9 max-w-xs"
                />
                <Button
                  type="button"
                  size="sm"
                  onClick={handleSaveContact}
                  disabled={!contactName.trim() || savingContact}
                >
                  Salvar contato
                </Button>
              </div>
            </div>

            <div className="flex-1 overflow-y-auto px-6 py-6">
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
                  Nenhuma mensagem ainda. Envie o primeiro SMS!
                </div>
              )}

              {!loadingChat && messages.length > 0 && (
                <div className="space-y-3">
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
                            "max-w-[70%] rounded-2xl px-4 py-2 text-sm shadow-sm",
                            outbound
                              ? "bg-blue-500 text-white"
                              : "bg-background text-foreground border",
                          )}
                        >
                          <p>{message.body}</p>
                          <div
                            className={cn(
                              "mt-1 text-[10px]",
                              outbound
                                ? "text-blue-100"
                                : "text-muted-foreground",
                            )}
                          >
                            {formatTime(message.created_at)}
                          </div>
                        </div>
                      </div>
                    );
                  })}
                  <div ref={messagesEndRef} />
                </div>
              )}
            </div>

            <div className="border-t bg-background px-6 py-4">
              <div className="flex flex-wrap items-center gap-3">
                <div className="flex items-center gap-2 text-xs text-muted-foreground">
                  <span>Remetente:</span>
                  <Input value="Conecta" disabled className="h-8 w-28" />
                </div>
                <span className="ml-auto text-xs text-muted-foreground">
                  {messageText.length}/160
                </span>
              </div>
              <div className="mt-3 flex items-center gap-2">
                <Input
                  value={messageText}
                  onChange={(event) => setMessageText(event.target.value)}
                  placeholder="Digite um SMS"
                  maxLength={160}
                  className="h-12"
                />
                <Button
                  type="button"
                  size="icon"
                  disabled={!messageText.trim() || sending}
                  onClick={handleSendMessage}
                  className="h-12 w-12 rounded-full"
                >
                  <Send size={18} />
                </Button>
              </div>
            </div>
          </>
        ) : (
          <div className="flex flex-1 flex-col items-center justify-center text-center">
            <div className="flex h-16 w-16 items-center justify-center rounded-full bg-blue-100 text-blue-600">
              <MessageSquare size={24} />
            </div>
            <h2 className="mt-4 text-lg font-semibold">Mensagens SMS</h2>
            <p className="mt-1 text-sm text-muted-foreground">
              Selecione um contato a esquerda ou inicie uma nova conversa para
              enviar um SMS.
            </p>
          </div>
        )}
      </section>
    </div>
  );
};

export default SmsPage;
