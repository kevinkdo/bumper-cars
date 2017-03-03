var wsUri = "ws://127.0.0.1:8000/";

const Playground = React.createClass({
  websocket: new WebSocket(wsUri),

  componentDidMount: function() {
    var me = this;
    me.setupKeyBindings();
    
    me.websocket.onopen = function(evt) { me.onWsOpen(evt) };
    me.websocket.onclose = function(evt) { me.onWsClose(evt) };
    me.websocket.onmessage = function(evt) { me.onWsMessage(evt) };
    me.websocket.onerror = function(evt) { me.onWsError(evt) };
  },

  sendUpdate: function(id, cars) {
    var update_message = {
      id: id,
      position: cars[id]
    };
    this.websocket.send(JSON.stringify(update_message));
  },

  setupKeyBindings: function() {
    var me = this;
    $(document).keydown(function(e) {
      switch(e.which) {
        case 37: // left
          me.setState((state) => me.state.cars[me.state.id].x -= 5);
        break;

        case 38: // up
          me.setState((state) => me.state.cars[me.state.id].y -= 5);
        break;

        case 39: // right
          me.setState((state) => me.state.cars[me.state.id].x += 5)
        break;

        case 40: // down
          me.setState((state) => me.state.cars[me.state.id].y += 5)
        break;

        default: return; // exit this handler for other keys
      }

      me.sendUpdate(me.state.id, me.state.cars);
      e.preventDefault(); // prevent the default action (scroll / move caret)
    });
  },

  onWsOpen: function() {
    this.setState({
      status: "Successfully connected to server."
    });
    this.websocket.send("getid");
  },

  onWsMessage: function(evt) {
    if (evt.data.startsWith("getid")) {
      const id = evt.data.slice(5);
      var cars = this.state.cars;
      cars[id] = {
        x: window.innerWidth / 2,
        y: window.innerHeight / 2
      }
      this.setState({
        id: id,
        cars: cars
      });
      this.sendUpdate(id, cars);
    } else if (evt.data.startsWith("deleteid")) {
      const id = evt.data.slice(8);
      var cars = this.state.cars;
      this.setState({
        cars: cars
      });
    } else {
      var message = JSON.parse(evt.data);
      if (message.id === this.state.id) {
        return
      }

      var cars = this.state.cars;
      cars[message.id] = message.position;
      this.setState({
        cars: cars
      });
    }
  },

  onWsClose: function(evt) {
    this.setState({status: "No connection to server."});
  },

  onWsError: function(evt) {
    alert("Error connecting to server: " + evt.data)
  },

  getInitialState: function() {
    var me = this;

    return {
      id: null,
      cars: {}, // id -> {x:, y:} objects
      status: "Setting up connection to server..."
    };
  },

  render: function() {
    var me = this;
    if (!this.state.id) {
      return <div>{this.state.status}</div>;
    }

    const circles = Object.keys(this.state.cars).map(function(key) {
      var position = me.state.cars[key];
      return <circle r="5" fill="black" cx={position.x} cy={position.y} key={"car" + key} />
    });

    return (
      <div>{this.state.status}
        <svg className="playground" width={window.innerWidth} height={window.innerHeight}>
          {circles}
        </svg>
      </div>
    );
  }
});


ReactDOM.render(<Playground />, document.getElementById('react_main'));