// package: dicebot
// file: dicebot.proto

import * as jspb from "google-protobuf";

export class GetVariableRequest extends jspb.Message {
  getUserId(): string;
  setUserId(value: string): void;

  getRoomId(): string;
  setRoomId(value: string): void;

  getVariableName(): string;
  setVariableName(value: string): void;

  serializeBinary(): Uint8Array;
  toObject(includeInstance?: boolean): GetVariableRequest.AsObject;
  static toObject(includeInstance: boolean, msg: GetVariableRequest): GetVariableRequest.AsObject;
  static extensions: {[key: number]: jspb.ExtensionFieldInfo<jspb.Message>};
  static extensionsBinary: {[key: number]: jspb.ExtensionFieldBinaryInfo<jspb.Message>};
  static serializeBinaryToWriter(message: GetVariableRequest, writer: jspb.BinaryWriter): void;
  static deserializeBinary(bytes: Uint8Array): GetVariableRequest;
  static deserializeBinaryFromReader(message: GetVariableRequest, reader: jspb.BinaryReader): GetVariableRequest;
}

export namespace GetVariableRequest {
  export type AsObject = {
    userId: string,
    roomId: string,
    variableName: string,
  }
}

export class GetVariableReply extends jspb.Message {
  getValue(): number;
  setValue(value: number): void;

  serializeBinary(): Uint8Array;
  toObject(includeInstance?: boolean): GetVariableReply.AsObject;
  static toObject(includeInstance: boolean, msg: GetVariableReply): GetVariableReply.AsObject;
  static extensions: {[key: number]: jspb.ExtensionFieldInfo<jspb.Message>};
  static extensionsBinary: {[key: number]: jspb.ExtensionFieldBinaryInfo<jspb.Message>};
  static serializeBinaryToWriter(message: GetVariableReply, writer: jspb.BinaryWriter): void;
  static deserializeBinary(bytes: Uint8Array): GetVariableReply;
  static deserializeBinaryFromReader(message: GetVariableReply, reader: jspb.BinaryReader): GetVariableReply;
}

export namespace GetVariableReply {
  export type AsObject = {
    value: number,
  }
}

export class GetAllVariablesRequest extends jspb.Message {
  getUserId(): string;
  setUserId(value: string): void;

  getRoomId(): string;
  setRoomId(value: string): void;

  serializeBinary(): Uint8Array;
  toObject(includeInstance?: boolean): GetAllVariablesRequest.AsObject;
  static toObject(includeInstance: boolean, msg: GetAllVariablesRequest): GetAllVariablesRequest.AsObject;
  static extensions: {[key: number]: jspb.ExtensionFieldInfo<jspb.Message>};
  static extensionsBinary: {[key: number]: jspb.ExtensionFieldBinaryInfo<jspb.Message>};
  static serializeBinaryToWriter(message: GetAllVariablesRequest, writer: jspb.BinaryWriter): void;
  static deserializeBinary(bytes: Uint8Array): GetAllVariablesRequest;
  static deserializeBinaryFromReader(message: GetAllVariablesRequest, reader: jspb.BinaryReader): GetAllVariablesRequest;
}

export namespace GetAllVariablesRequest {
  export type AsObject = {
    userId: string,
    roomId: string,
  }
}

export class GetAllVariablesReply extends jspb.Message {
  getVariablesMap(): jspb.Map<string, number>;
  clearVariablesMap(): void;
  serializeBinary(): Uint8Array;
  toObject(includeInstance?: boolean): GetAllVariablesReply.AsObject;
  static toObject(includeInstance: boolean, msg: GetAllVariablesReply): GetAllVariablesReply.AsObject;
  static extensions: {[key: number]: jspb.ExtensionFieldInfo<jspb.Message>};
  static extensionsBinary: {[key: number]: jspb.ExtensionFieldBinaryInfo<jspb.Message>};
  static serializeBinaryToWriter(message: GetAllVariablesReply, writer: jspb.BinaryWriter): void;
  static deserializeBinary(bytes: Uint8Array): GetAllVariablesReply;
  static deserializeBinaryFromReader(message: GetAllVariablesReply, reader: jspb.BinaryReader): GetAllVariablesReply;
}

export namespace GetAllVariablesReply {
  export type AsObject = {
    variablesMap: Array<[string, number]>,
  }
}

export class SetVariableRequest extends jspb.Message {
  getUserId(): string;
  setUserId(value: string): void;

  getRoomId(): string;
  setRoomId(value: string): void;

  getVariableName(): string;
  setVariableName(value: string): void;

  getValue(): number;
  setValue(value: number): void;

  serializeBinary(): Uint8Array;
  toObject(includeInstance?: boolean): SetVariableRequest.AsObject;
  static toObject(includeInstance: boolean, msg: SetVariableRequest): SetVariableRequest.AsObject;
  static extensions: {[key: number]: jspb.ExtensionFieldInfo<jspb.Message>};
  static extensionsBinary: {[key: number]: jspb.ExtensionFieldBinaryInfo<jspb.Message>};
  static serializeBinaryToWriter(message: SetVariableRequest, writer: jspb.BinaryWriter): void;
  static deserializeBinary(bytes: Uint8Array): SetVariableRequest;
  static deserializeBinaryFromReader(message: SetVariableRequest, reader: jspb.BinaryReader): SetVariableRequest;
}

export namespace SetVariableRequest {
  export type AsObject = {
    userId: string,
    roomId: string,
    variableName: string,
    value: number,
  }
}

export class SetVariableReply extends jspb.Message {
  getSuccess(): boolean;
  setSuccess(value: boolean): void;

  serializeBinary(): Uint8Array;
  toObject(includeInstance?: boolean): SetVariableReply.AsObject;
  static toObject(includeInstance: boolean, msg: SetVariableReply): SetVariableReply.AsObject;
  static extensions: {[key: number]: jspb.ExtensionFieldInfo<jspb.Message>};
  static extensionsBinary: {[key: number]: jspb.ExtensionFieldBinaryInfo<jspb.Message>};
  static serializeBinaryToWriter(message: SetVariableReply, writer: jspb.BinaryWriter): void;
  static deserializeBinary(bytes: Uint8Array): SetVariableReply;
  static deserializeBinaryFromReader(message: SetVariableReply, reader: jspb.BinaryReader): SetVariableReply;
}

export namespace SetVariableReply {
  export type AsObject = {
    success: boolean,
  }
}

export class UserIdRequest extends jspb.Message {
  getUserId(): string;
  setUserId(value: string): void;

  serializeBinary(): Uint8Array;
  toObject(includeInstance?: boolean): UserIdRequest.AsObject;
  static toObject(includeInstance: boolean, msg: UserIdRequest): UserIdRequest.AsObject;
  static extensions: {[key: number]: jspb.ExtensionFieldInfo<jspb.Message>};
  static extensionsBinary: {[key: number]: jspb.ExtensionFieldBinaryInfo<jspb.Message>};
  static serializeBinaryToWriter(message: UserIdRequest, writer: jspb.BinaryWriter): void;
  static deserializeBinary(bytes: Uint8Array): UserIdRequest;
  static deserializeBinaryFromReader(message: UserIdRequest, reader: jspb.BinaryReader): UserIdRequest;
}

export namespace UserIdRequest {
  export type AsObject = {
    userId: string,
  }
}

export class RoomsListReply extends jspb.Message {
  clearRoomsList(): void;
  getRoomsList(): Array<RoomsListReply.Room>;
  setRoomsList(value: Array<RoomsListReply.Room>): void;
  addRooms(value?: RoomsListReply.Room, index?: number): RoomsListReply.Room;

  serializeBinary(): Uint8Array;
  toObject(includeInstance?: boolean): RoomsListReply.AsObject;
  static toObject(includeInstance: boolean, msg: RoomsListReply): RoomsListReply.AsObject;
  static extensions: {[key: number]: jspb.ExtensionFieldInfo<jspb.Message>};
  static extensionsBinary: {[key: number]: jspb.ExtensionFieldBinaryInfo<jspb.Message>};
  static serializeBinaryToWriter(message: RoomsListReply, writer: jspb.BinaryWriter): void;
  static deserializeBinary(bytes: Uint8Array): RoomsListReply;
  static deserializeBinaryFromReader(message: RoomsListReply, reader: jspb.BinaryReader): RoomsListReply;
}

export namespace RoomsListReply {
  export type AsObject = {
    roomsList: Array<RoomsListReply.Room.AsObject>,
  }

  export class Room extends jspb.Message {
    getRoomId(): string;
    setRoomId(value: string): void;

    getDisplayName(): string;
    setDisplayName(value: string): void;

    serializeBinary(): Uint8Array;
    toObject(includeInstance?: boolean): Room.AsObject;
    static toObject(includeInstance: boolean, msg: Room): Room.AsObject;
    static extensions: {[key: number]: jspb.ExtensionFieldInfo<jspb.Message>};
    static extensionsBinary: {[key: number]: jspb.ExtensionFieldBinaryInfo<jspb.Message>};
    static serializeBinaryToWriter(message: Room, writer: jspb.BinaryWriter): void;
    static deserializeBinary(bytes: Uint8Array): Room;
    static deserializeBinaryFromReader(message: Room, reader: jspb.BinaryReader): Room;
  }

  export namespace Room {
    export type AsObject = {
      roomId: string,
      displayName: string,
    }
  }
}

