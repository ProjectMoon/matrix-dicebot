// package: web_api
// file: web-api.proto

var web_api_pb = require("./web-api_pb");
var grpc = require("@improbable-eng/grpc-web").grpc;

var WebApi = (function () {
  function WebApi() {}
  WebApi.serviceName = "web_api.WebApi";
  return WebApi;
}());

WebApi.ListRoom = {
  methodName: "ListRoom",
  service: WebApi,
  requestStream: false,
  responseStream: false,
  requestType: web_api_pb.UserIdRequest,
  responseType: web_api_pb.RoomsListReply
};

exports.WebApi = WebApi;

function WebApiClient(serviceHost, options) {
  this.serviceHost = serviceHost;
  this.options = options || {};
}

WebApiClient.prototype.listRoom = function listRoom(requestMessage, metadata, callback) {
  if (arguments.length === 2) {
    callback = arguments[1];
  }
  var client = grpc.unary(WebApi.ListRoom, {
    request: requestMessage,
    host: this.serviceHost,
    metadata: metadata,
    transport: this.options.transport,
    debug: this.options.debug,
    onEnd: function (response) {
      if (callback) {
        if (response.status !== grpc.Code.OK) {
          var err = new Error(response.statusMessage);
          err.code = response.status;
          err.metadata = response.trailers;
          callback(err, null);
        } else {
          callback(null, response.message);
        }
      }
    }
  });
  return {
    cancel: function () {
      callback = null;
      client.close();
    }
  };
};

exports.WebApiClient = WebApiClient;

