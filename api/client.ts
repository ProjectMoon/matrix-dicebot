import { grpc } from "@improbable-eng/grpc-web";

// Import code-generated data structures.
import { WebApi, WebApiClient } from "./_proto/web-api_pb_service";
import { UserIdRequest, RoomsListReply } from "./_proto/web-api_pb";

const listRoomsRequest = new UserIdRequest();
listRoomsRequest.setUserId("@projectmoon:agnos.is");
grpc.unary(WebApi.ListRoom, {
    request: listRoomsRequest,
    host: "http://localhost:10000",
    onEnd: res => {
        const { status, statusMessage, headers, message, trailers } = res;
        console.log(status, '-', statusMessage);
        if (status === grpc.Code.OK && message) {
            console.log("all ok. got rooms: ", message.toObject());
        }
    }
});
