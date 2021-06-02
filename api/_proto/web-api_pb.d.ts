// package: web_api
// file: web-api.proto

import * as jspb from "google-protobuf";

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

