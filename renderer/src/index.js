import React from 'react';
import ReactDOMServer from 'react-dom/server';

import Component from '__SSR_BUNDLE__';

export default (props) => {
  return ReactDOMServer.renderToString(<Component {...props} />);
};
