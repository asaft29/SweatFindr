export enum UserRole {
  GUEST = "guest",
  CLIENT = "client",
  OWNER_EVENT = "owner-event",
}

export interface User {
  id: number;
  email: string;
  role: UserRole;
  emailVerified: boolean;
}

export interface AuthState {
  user: User | null;
  token: string | null;
  isAuthenticated: boolean;
  isLoading: boolean;
}

export interface LoginRequest {
  username: string;
  password: string;
}

export interface LoginResponse {
  success: boolean;
  message: string;
  token_value?: string;
}

export interface RegisterRequest {
  email: string;
  password: string;
  role: string;
}

export interface RegisterResponse {
  success: boolean;
  message: string;
  user_id?: number;
  token_value?: string;
}

export interface LogoutRequest {
  token_value: string;
}

export interface LogoutResponse {
  success: boolean;
  message: string;
}

export interface VerifyEmailRequest {
  user_id: number;
  verification_code: string;
}

export interface VerifyEmailResponse {
  success: boolean;
  message: string;
}

export interface ResendVerificationRequest {
  user_id: number;
  email: string;
}

export interface ResendVerificationResponse {
  success: boolean;
  message: string;
}

export interface Event {
  id: number;
  id_owner: number;
  nume: string;
  locatie: string | null;
  descriere: string | null;
  numarlocuri: number | null;
}

export interface EventWithLinks extends Event {
  _links?: HateoasLinks;
}

export interface EventPackage {
  id: number;
  id_owner: number;
  nume: string;
  locatie: string | null;
  descriere: string | null;
  numarlocuri: number | null;
}

export interface CreateEventRequest {
  nume: string;
  locatie?: string;
  descriere?: string;
  numarlocuri?: number;
}

export interface UpdateEventRequest {
  nume: string;
  locatie?: string;
  descriere?: string;
  numarlocuri?: number;
}

export interface CreatePackageRequest {
  nume: string;
  locatie?: string;
  descriere?: string;
}

export interface UpdatePackageRequest {
  nume: string;
  locatie?: string;
  descriere?: string;
}

export interface SocialMedia {
  instagram?: string;
  facebook?: string;
  twitter?: string;
  linkedin?: string;
  github?: string;
  public?: boolean;
}

export interface TicketRef {
  cod: string;
  nume_eveniment?: string;
  locatie?: string;
  descriere?: string;
  refund_status?: string;
}

export interface Client {
  _id: string;
  email: string;
  prenume: string;
  nume: string;
  public_info?: boolean;
  social_media?: SocialMedia;
  lista_bilete?: TicketRef[];
}

export interface TicketBuyerInfo {
  email: string;
  prenume?: string;
  nume?: string;
  public_info: boolean;
}

export interface HateoasLink {
  href: string;
  type?: string;
}

export interface HateoasLinks {
  self: HateoasLink;
  parent?: HateoasLink;
  next?: HateoasLink;
  prev?: HateoasLink;
  [key: string]: HateoasLink | undefined;
}

export interface EventPackageWithLinks extends EventPackage {
  _links?: HateoasLinks;
}

export type RefundStatus =
  | "Pending"
  | "Approved"
  | "Rejected"
  | "PENDING"
  | "APPROVED"
  | "REJECTED";

export interface RefundRequest {
  id: number;
  ticket_cod: string;
  requester_id: number;
  requester_email: string;
  event_id?: number;
  packet_id?: number;
  event_owner_id: number;
  reason: string;
  status: RefundStatus;
  rejection_message?: string;
  created_at?: string;
  resolved_at?: string;
}

export interface CreateRefundRequest {
  ticket_cod: string;
  reason: string;
}

export interface RefundResponse {
  message: string;
}
