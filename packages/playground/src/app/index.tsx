/* eslint-disable @typescript-eslint/no-explicit-any */
import { Console } from './Console';
import { InkEditor } from '@paritytech/ink-editor';
import { Layout } from './Layout';
import { Header } from './Header';
import { AppContext, AppProvider } from '~/context/app/';
import { MessageContext, MessageProvider } from '~/context/messages/';
import { ReactElement, useContext, useEffect, useState} from 'react';
import { Dispatch, State } from '~/context/app/reducer';
import { MessageDispatch, MessageState } from '~/context/messages/reducer';
import { loadCode } from '~/context/side-effects/load-code';
import { monaco } from 'react-monaco-editor';
import {
  Routes,
  Route,
  useLocation
} from "react-router-dom";

const useInkVersions = () => {
  const [versions, setVersions] = useState<any>([]);
  useEffect(() => {
    fetch('https://crates.io/api/v1/crates/ink')
    .then((res) => res.json())
    .then((res) => {
      const inkVersions = res.versions.map((d: any) => {
        return { ink_version: d.num, name: `v${d.num}` }
      }).filter((f: any) => f.ink_version.length < 7).splice(0, 5);
      if (inkVersions && Array.isArray(inkVersions)) {
        setVersions(inkVersions);
      }
    })
  }, []);
  return versions
}

const App = (): ReactElement => {
  const inkVersions = useInkVersions();
  const [state, dispatch]: [State, Dispatch] = useContext(AppContext);
  const [, messageDispatch]: [MessageState, MessageDispatch] = useContext(MessageContext);
  const location = useLocation();
  const { monacoUri: uri, formatting } = state;

  useEffect(() => {
    if (!uri) return;
    loadCode(state, { app: dispatch, message: messageDispatch }).then(code => {
      const model = monaco.editor.getModel(uri as monaco.Uri);
      if (!model) return;
      model.setValue(code);
    });
  }, [uri]);

  useEffect(() => {
    if (!(formatting.type === 'RESULT')) return;
    if (!(formatting.payload.type === 'OK')) return;
    if (!(formatting.payload.payload.type === 'SUCCESS')) return;
    if (!uri) return;
    const model = monaco.editor.getModel(uri as monaco.Uri);
    if (!model) return;
    const code = formatting.payload.payload.payload.source;
    model.setValue(code);
  }, [formatting]);
  
  useEffect(() => {
    if (location.pathname) {
      if (location.pathname.length > 6 && location.pathname.length < 9) {
        dispatch({
          type: 'SET_INK_VERSION',
          payload: location.pathname.slice(2, 7),
        });
      }
    }
  }, [location.pathname])

  const onRustAnalyzerStartLoad = () => {
    messageDispatch({
      type: 'LOG_SYSTEM',
      payload: { status: 'IN_PROGRESS', content: 'Loading Rust Analyzer...' },
    });
  };

  const onRustAnalyzerFinishLoad = () => {
    dispatch({
      type: 'SET_RUST_ANALYZER_STATE',
      payload: true,
    });
    messageDispatch({
      type: 'LOG_SYSTEM',
      payload: { status: 'DONE', content: 'Rust Analyzer Ready' },
    });
  };

  if (!state.ink_version) {
    return <div>Hello</div>
  }
  
  return (
    <Layout
      key={state.ink_version}
      header={<Header inkVersions={inkVersions} />}
      editor={
        <InkEditor
          onRustAnalyzerStartLoad={onRustAnalyzerStartLoad}
          onRustAnalyzerFinishLoad={onRustAnalyzerFinishLoad}
          numbering={state.numbering}
          darkmode={state.darkmode}
          rustAnalyzer={state.rustAnalyzer}
          minimap={state.minimap}
          setURI={uri => dispatch({ type: 'SET_URI', payload: uri })}
          selectedInkVersion={state.ink_version}
        />
      }
      console={<Console />}
    />
  );
};

const AppWithProvider = (): ReactElement => {
  return (
    <AppProvider>
      <MessageProvider>
        <Routes>
          <Route path="/" element={<App />} />
          <Route path="/:versionId/" element={<App />} />
        </Routes>
      </MessageProvider>
    </AppProvider>
  );
};

export default AppWithProvider;
