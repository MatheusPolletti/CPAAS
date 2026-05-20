import z from "zod";

export const LoginFormSchema = z.object({
  email: z.email({
    message: "Coloque um endereço de e-mail válido.",
  }),
  password: z
    .string()
    .min(1, {
      message: "A senha precisa de no mínimo 1 caracter.",
    })
    .max(1024, {
      message: "A senha não pode ter mais que 1024 caracteres.",
    }),
});