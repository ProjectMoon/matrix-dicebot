// package: dicebot
// file: dicebot.proto

import * as dicebot_pb from "./dicebot_pb";
import {grpc} from "@improbable-eng/grpc-web";

type DicebotGetVariable = {
  readonly methodName: string;
  readonly service: typeof Dicebot;
  readonly requestStream: false;
  readonly responseStream: false;
  readonly requestType: typeof dicebot_pb.GetVariableRequest;
  readonly responseType: typeof dicebot_pb.GetVariableReply;
};

type DicebotGetAllVariables = {
  readonly methodName: string;
  readonly service: typeof Dicebot;
  readonly requestStream: false;
  readonly responseStream: false;
  readonly requestType: typeof dicebot_pb.GetAllVariablesRequest;
  readonly responseType: typeof dicebot_pb.GetAllVariablesReply;
};

type DicebotSetVariable = {
  readonly methodName: string;
  readonly service: typeof Dicebot;
  readonly requestStream: false;
  readonly responseStream: false;
  readonly requestType: typeof dicebot_pb.SetVariableRequest;
  readonly responseType: typeof dicebot_pb.SetVariableReply;
};

type DicebotRoomsForUser = {
  readonly methodName: string;
  readonly service: typeof Dicebot;
  readonly requestStream: false;
  readonly responseStream: false;
  readonly requestType: typeof dicebot_pb.UserIdRequest;
  readonly responseType: typeof dicebot_pb.RoomsListReply;
};

export class Dicebot {
  static readonly serviceName: string;
  static readonly GetVariable: DicebotGetVariable;
  static readonly GetAllVariables: DicebotGetAllVariables;
  static readonly SetVariable: DicebotSetVariable;
  static readonly RoomsForUser: DicebotRoomsForUser;
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

export class DicebotClient {
  readonly serviceHost: string;

  constructor(serviceHost: string, options?: grpc.RpcOptions);
  getVariable(
    requestMessage: dicebot_pb.GetVariableRequest,
    metadata: grpc.Metadata,
    callback: (error: ServiceError|null, responseMessage: dicebot_pb.GetVariableReply|null) => void
  ): UnaryResponse;
  getVariable(
    requestMessage: dicebot_pb.GetVariableRequest,
    callback: (error: ServiceError|null, responseMessage: dicebot_pb.GetVariableReply|null) => void
  ): UnaryResponse;
  getAllVariables(
    requestMessage: dicebot_pb.GetAllVariablesRequest,
    metadata: grpc.Metadata,
    callback: (error: ServiceError|null, responseMessage: dicebot_pb.GetAllVariablesReply|null) => void
  ): UnaryResponse;
  getAllVariables(
    requestMessage: dicebot_pb.GetAllVariablesRequest,
    callback: (error: ServiceError|null, responseMessage: dicebot_pb.GetAllVariablesReply|null) => void
  ): UnaryResponse;
  setVariable(
    requestMessage: dicebot_pb.SetVariableRequest,
    metadata: grpc.Metadata,
    callback: (error: ServiceError|null, responseMessage: dicebot_pb.SetVariableReply|null) => void
  ): UnaryResponse;
  setVariable(
    requestMessage: dicebot_pb.SetVariableRequest,
    callback: (error: ServiceError|null, responseMessage: dicebot_pb.SetVariableReply|null) => void
  ): UnaryResponse;
  roomsForUser(
    requestMessage: dicebot_pb.UserIdRequest,
    metadata: grpc.Metadata,
    callback: (error: ServiceError|null, responseMessage: dicebot_pb.RoomsListReply|null) => void
  ): UnaryResponse;
  roomsForUser(
    requestMessage: dicebot_pb.UserIdRequest,
    callback: (error: ServiceError|null, responseMessage: dicebot_pb.RoomsListReply|null) => void
  ): UnaryResponse;
}

