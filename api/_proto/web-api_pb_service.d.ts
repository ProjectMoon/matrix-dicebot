// package: web_api
// file: web-api.proto

import * as web_api_pb from "./web-api_pb";
import {grpc} from "@improbable-eng/grpc-web";

type WebApiListRoom = {
  readonly methodName: string;
  readonly service: typeof WebApi;
  readonly requestStream: false;
  readonly responseStream: false;
  readonly requestType: typeof web_api_pb.UserIdRequest;
  readonly responseType: typeof web_api_pb.RoomsListReply;
};

export class WebApi {
  static readonly serviceName: string;
  static readonly ListRoom: WebApiListRoom;
}

export type ServiceError = { message: string, code: number; metadata: grpc.Metadata }
export type Status = { details: string, code: number; metadata: grpc.Metadata }

interface UnaryResponse {
  cancel(): void;
}
interface ResponseStream<T> {
  cancel(): void;
  on(type: 'data', handler: (message: T) => void): ResponseStream<T>;
  on(type: 'end', handler: (status?: Status) => void): ResponseStream<T>;
  on(type: 'status', handler: (status: Status) => void): ResponseStream<T>;
}
interface RequestStream<T> {
  write(message: T): RequestStream<T>;
  end(): void;
  cancel(): void;
  on(type: 'end', handler: (status?: Status) => void): RequestStream<T>;
  on(type: 'status', handler: (status: Status) => void): RequestStream<T>;
}
interface BidirectionalStream<ReqT, ResT> {
  write(message: ReqT): BidirectionalStream<ReqT, ResT>;
  end(): void;
  cancel(): void;
  on(type: 'data', handler: (message: ResT) => void): BidirectionalStream<ReqT, ResT>;
  on(type: 'end', handler: (status?: Status) => void): BidirectionalStream<ReqT, ResT>;
  on(type: 'status', handler: (status: Status) => void): BidirectionalStream<ReqT, ResT>;
}

export class WebApiClient {
  readonly serviceHost: string;

  constructor(serviceHost: string, options?: grpc.RpcOptions);
  listRoom(
    requestMessage: web_api_pb.UserIdRequest,
    metadata: grpc.Metadata,
    callback: (error: ServiceError|null, responseMessage: web_api_pb.RoomsListReply|null) => void
  ): UnaryResponse;
  listRoom(
    requestMessage: web_api_pb.UserIdRequest,
    callback: (error: ServiceError|null, responseMessage: web_api_pb.RoomsListReply|null) => void
  ): UnaryResponse;
}

