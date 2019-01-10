import React from 'react';
import styled from 'styled-components';

const SIZE = '200px';

const M = styled.div`
  position: relative;
  
  height: calc(${SIZE} /2);
  width: ${SIZE};
  
  overflow: hidden;
`;

const Background = styled.div`
  position: absolute;
  width: 100%;
  height: 100%;
  border: calc(${SIZE}/8) solid #CCC;
  border-bottom: none;
  box-sizing: border-box;
  border-radius: 50% 50% 50% 50%/100% 100% 0% 0%;
`;

const NeedleWrapper = styled.div`
  position: absolute;
  bottom: 0;
  left: 0;

  height: calc(${SIZE} /2);
  width: 100%;
  
  transition: transform 1s;
  transform-origin: center bottom;
  transform: rotate(${props => props.percent}turn);
`;

const Needle = styled.div`
  width: 2px;
  background: black;
  height: 70%;
  margin: 0 auto;
`;

const Text = styled.div`
  position: absolute;
  bottom: 0;
  left: 0;
  right: 0;
  
  font-weight: bold;
  text-align: center;
`;

export default function Meter(props) {
  const percent = props.value / 50000000 / 2 - 0.25;
  return <M><Background /><NeedleWrapper percent={percent}><Needle /></NeedleWrapper><Text>{props.value}</Text></M>;
}