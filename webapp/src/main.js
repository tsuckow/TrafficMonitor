import React from "react";
import ReactDOM from "react-dom";

import { Poll } from 'restful-react';

import Meter from './meter.js'

const Polled = () => <Poll path="api">{
  (data) => <div>
  Average Bps
  <Meter value={data && data.avg_bytes_per_second || 0} />
  <br/><br/>
  Last Second Bps ({data && (data.last_second_bytes/1000/1000 * 8).toFixed(2) || 0} Mbps)
  <Meter value={data && data.last_second_bytes || 0} />
  </div>
}</Poll>;

const App = () => <div><Polled/></div>;

ReactDOM.render(
  <App />,
  document.getElementById('app')
);