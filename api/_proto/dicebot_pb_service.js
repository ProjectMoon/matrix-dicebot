// package: dicebot
// file: dicebot.proto

var dicebot_pb = require("./dicebot_pb");
var grpc = require("@improbable-eng/grpc-web").grpc;

var Dicebot = (function () {
  function Dicebot() {}
  Dicebot.serviceName = "dicebot.Dicebot";
  return Dicebot;
}());

Dicebot.GetVariable = {
  methodName: "GetVariable",
  service: Dicebot,
  requestStream: false,
  responseStream: false,
  requestType: dicebot_pb.GetVariableRequest,
  responseType: dicebot_pb.GetVariableReply
};

Dicebot.GetAllVariables = {
  methodName: "GetAllVariables",
  service: Dicebot,
  requestStream: false,
  responseStream: false,
  requestType: dicebot_pb.GetAllVariablesRequest,
  responseType: dicebot_pb.GetAllVariablesReply
};

Dicebot.SetVariable = {
  methodName: "SetVariable",
  service: Dicebot,
  requestStream: false,
  responseStream: false,
  requestType: dicebot_pb.SetVariableRequest,
  responseType: dicebot_pb.SetVariableReply
};

Dicebot.RoomsForUser = {
  methodName: "RoomsForUser",
  service: Dicebot,
  requestStream: false,
  responseStream: false,
  requestType: dicebot_pb.UserIdRequest,
  responseType: dicebot_pb.RoomsListReply
};

exports.Dicebot = Dicebot;

function DicebotClient(serviceHost, options) {
  this.serviceHost = serviceHost;
  this.options = options || {};
}

DicebotClient.prototype.getVariable = function getVariable(requestMessage, metadata, callback) {
  if (arguments.length === 2) {
    callback = arguments[1];
  }
  var client = grpc.unary(Dicebot.GetVariable, {
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

DicebotClient.prototype.getAllVariables = function getAllVariables(requestMessage, metadata, callback) {
  if (arguments.length === 2) {
    callback = arguments[1];
  }
  var client = grpc.unary(Dicebot.GetAllVariables, {
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

DicebotClient.prototype.setVariable = function setVariable(requestMessage, metadata, callback) {
  if (arguments.length === 2) {
    callback = arguments[1];
  }
  var client = grpc.unary(Dicebot.SetVariable, {
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

DicebotClient.prototype.roomsForUser = function roomsForUser(requestMessage, metadata, callback) {
  if (arguments.length === 2) {
    callback = arguments[1];
  }
  var client = grpc.unary(Dicebot.RoomsForUser, {
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

exports.DicebotClient = DicebotClient;

