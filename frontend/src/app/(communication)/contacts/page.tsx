"use client";

import { useCallback, useEffect, useMemo, useState } from "react";
import { useSession } from "next-auth/react";
import { toast } from "sonner";
import { useAxiosAuth } from "@/lib/axios-auth";
import { Button } from "@/components/ui/button";
import {
  Dialog,
  DialogClose,
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogTitle,
  DialogTrigger,
} from "@/components/ui/dialog";
import { Input } from "@/components/ui/input";
import { Separator } from "@/components/ui/separator";
import { Skeleton } from "@/components/ui/skeleton";
import { Plus, Search, Users } from "lucide-react";

type Contact = {
  id: number;
  phone_number: string;
  name: string;
  company?: string | null;
};

const ContactsPage = () => {
  const axiosAuth = useAxiosAuth();
  const { status } = useSession();

  const [contacts, setContacts] = useState<Contact[]>([]);
  const [loading, setLoading] = useState(false);
  const [saving, setSaving] = useState(false);
  const [name, setName] = useState("");
  const [phone, setPhone] = useState("");
  const [company, setCompany] = useState("");
  const [search, setSearch] = useState("");
  const [dialogOpen, setDialogOpen] = useState(false);
  const [phoneError, setPhoneError] = useState<string | null>(null);
  const PHONE_MIN = 10;
  const PHONE_MAX = 14;

  const fetchContacts = useCallback(async () => {
    try {
      const response = await axiosAuth.get<Contact[]>("/contact/contacts");

      setContacts(response.data ?? []);
    } catch {
      toast.error("Erro ao buscar contatos", { position: "top-center" });
    }
  }, [axiosAuth]);

  useEffect(() => {
    if (status === "authenticated") {
      let isMounted = true;

      const initializeData = async () => {
        setLoading(true);
        await fetchContacts();
        if (isMounted) setLoading(false);
      };

      initializeData();

      return () => {
        isMounted = false;
      };
    }
  }, [status, fetchContacts]);

  const filteredContacts = useMemo(() => {
    const term = search.trim().toLowerCase();
    if (!term) return contacts;
    return contacts.filter((contact) => {
      return (
        contact.name.toLowerCase().includes(term) ||
        contact.phone_number.toLowerCase().includes(term) ||
        (contact.company?.toLowerCase().includes(term) ?? false)
      );
    });
  }, [contacts, search]);

  const sortedContacts = useMemo(() => {
    return [...filteredContacts].sort((a, b) => a.name.localeCompare(b.name));
  }, [filteredContacts]);

  const normalizePhone = (value: string) => {
    const digits = value.replace(/\D/g, "");
    return `+${digits}`;
  };

  const getErrorMessage = (error: unknown) => {
    if (typeof error !== "object" || error === null) {
      return "Erro ao salvar contato";
    }

    const response = "response" in error ? error.response : null;
    if (
      response &&
      typeof response === "object" &&
      response !== null &&
      "data" in response
    ) {
      const data = response.data;
      if (typeof data === "string" && data.trim()) {
        return data;
      }

      if (
        typeof data === "object" &&
        data !== null &&
        "message" in data &&
        typeof data.message === "string" &&
        data.message.trim()
      ) {
        return data.message;
      }
    }

    return "Erro ao salvar contato";
  };

  const isPhoneValid = (value: string) => {
    const digits = value.replace(/\D/g, "");
    return digits.length >= PHONE_MIN && digits.length <= PHONE_MAX;
  };

  const handleSave = async () => {
    const trimmedName = name.trim();
    const trimmedPhone = phone.trim();
    const trimmedCompany = company.trim();
    if (!trimmedName || !trimmedPhone || saving) return;

    if (!isPhoneValid(trimmedPhone)) {
      setPhoneError(
        `Informe um número válido com DDI e DDD (entre ${PHONE_MIN} e ${PHONE_MAX} dígitos).`,
      );
      return;
    }

    setSaving(true);
    try {
      await axiosAuth.post("/contact/save", {
        phone_number: normalizePhone(trimmedPhone),
        name: trimmedName,
        company: trimmedCompany || null,
      });

      setName("");
      setPhone("");
      setCompany("");
      setPhoneError(null);
      setDialogOpen(false);

      toast.success("Contato salvo", { position: "top-center" });
    } catch (error) {
      console.log("erro");
      toast.error(getErrorMessage(error), { position: "top-center" });
    } finally {
      setSaving(false);
    }
  };

  const handlePhoneChange = (value: string) => {
    const digits = value.replace(/\D/g, "").slice(0, PHONE_MAX);
    setPhone(`+${digits}`);
    if (phoneError) setPhoneError(null);
  };

  if (status === "loading") {
    return <p className="p-6 text-muted-foreground">Conectando...</p>;
  }

  return (
    <div className="flex h-svh w-full flex-col gap-6 bg-muted/30 p-6">
      <div className="flex items-center gap-3">
        <div className="flex h-10 w-10 items-center justify-center rounded-xl bg-indigo-100 text-indigo-600">
          <Users size={18} />
        </div>
        <div>
          <p className="text-base font-semibold">Contatos</p>
          <p className="text-sm text-muted-foreground">
            Lista de contatos salvos
          </p>
        </div>
      </div>

      <div className="flex flex-col gap-4 rounded-2xl border bg-background p-5 shadow-sm">
        <div className="flex flex-wrap items-center justify-between gap-3">
          <div>
            <p className="text-sm font-semibold">Contatos salvos</p>
            <p className="text-xs text-muted-foreground">
              Gerencie seus contatos por nome, telefone e companhia.
            </p>
          </div>
          <Dialog open={dialogOpen} onOpenChange={setDialogOpen}>
            <DialogTrigger asChild>
              <Button type="button" className="h-10 gap-2">
                <Plus size={16} />
                Adicionar contato
              </Button>
            </DialogTrigger>
            <DialogContent>
              <DialogHeader>
                <DialogTitle>Novo contato</DialogTitle>
                <DialogDescription>
                  Preencha os dados para salvar o contato.
                </DialogDescription>
              </DialogHeader>
              <div className="mt-4 flex flex-col gap-3">
                <div className="flex flex-col gap-2">
                  <label className="text-xs font-medium text-muted-foreground">
                    Nome
                  </label>
                  <Input
                    value={name}
                    onChange={(event) => setName(event.target.value)}
                    placeholder="Nome do contato"
                  />
                </div>
                <div className="flex flex-col gap-2">
                  <label className="text-xs font-medium text-muted-foreground">
                    Telefone
                  </label>
                  <Input
                    value={phone}
                    onChange={(event) => handlePhoneChange(event.target.value)}
                    placeholder="+55 11 99999-9999"
                    inputMode="numeric"
                    maxLength={PHONE_MAX + 1}
                    aria-invalid={!!phoneError}
                    className={phoneError ? "border-red-500" : undefined}
                  />
                  {phoneError && (
                    <span className="text-xs text-red-500">{phoneError}</span>
                  )}
                </div>
                <div className="flex flex-col gap-2">
                  <label className="text-xs font-medium text-muted-foreground">
                    Companhia
                  </label>
                  <Input
                    value={company}
                    onChange={(event) => setCompany(event.target.value)}
                    placeholder="Nome da companhia"
                  />
                </div>
              </div>
              <DialogFooter>
                <DialogClose asChild>
                  <Button type="button" variant="ghost">
                    Cancelar
                  </Button>
                </DialogClose>
                <Button
                  type="button"
                  onClick={handleSave}
                  disabled={!name.trim() || !phone.trim() || saving}
                >
                  Salvar
                </Button>
              </DialogFooter>
            </DialogContent>
          </Dialog>
        </div>

        <Separator />

        <div className="flex items-center gap-2">
          <Search size={16} className="text-muted-foreground" />
          <Input
            value={search}
            onChange={(event) => setSearch(event.target.value)}
            placeholder="Buscar contato"
            className="h-9"
          />
        </div>

        <Separator />

        <div className="flex flex-col">
          {loading && (
            <div className="space-y-3 py-2">
              {Array.from({ length: 4 }).map((_, index) => (
                <div key={index} className="flex items-center gap-3">
                  <Skeleton className="h-10 w-10 rounded-full" />
                  <div className="flex-1 space-y-2">
                    <Skeleton className="h-3 w-32" />
                    <Skeleton className="h-3 w-24" />
                  </div>
                </div>
              ))}
            </div>
          )}

          {!loading && sortedContacts.length === 0 && (
            <p className="text-sm text-muted-foreground">
              Nenhum contato salvo.
            </p>
          )}

          {!loading && sortedContacts.length > 0 && (
            <div className="divide-y">
              {sortedContacts.map((contact) => (
                <div key={contact.id} className="flex items-center py-3">
                  <div className="flex-1">
                    <p className="text-sm font-semibold">{contact.name}</p>
                    <p className="text-xs text-muted-foreground">
                      {contact.phone_number}
                    </p>
                    {contact.company && (
                      <p className="text-xs text-muted-foreground">
                        {contact.company}
                      </p>
                    )}
                  </div>
                </div>
              ))}
            </div>
          )}
        </div>
      </div>
    </div>
  );
};

export default ContactsPage;
