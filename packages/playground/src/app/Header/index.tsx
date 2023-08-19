/* eslint-disable @typescript-eslint/no-explicit-any */
import { ReactElement, useContext, useEffect, useRef, useState } from 'react';
import {
  Logo,
  CompileIcon,
  DownloadIcon,
  GithubRepoIcon,
  SettingsIcon,
  ShareIcon,
  TestingIcon,
  DocsIcon,
  DeployIcon,
  FormatIcon,
} from '~/symbols';
import { OverlayPanel, ButtonWithIcon } from '@paritytech/components/';
import { SettingsSubmenu } from './SettingsSubmenu';
import { ShareSubmenu } from './ShareSubmenu';

import { AppContext } from '~/context/app/';
import { MessageContext } from '~/context/messages/';
import { Dispatch, State } from '~/context/app/reducer';
import { MessageState, MessageDispatch, mapSizeInfo } from '~/context/messages/reducer';
import { compile } from '~/context/side-effects/compile';
import { testing } from '~/context/side-effects/testing';
import { format } from '~/context/side-effects/format';
import * as constants from '~/constants';
import { Colors } from '@paritytech/components/ButtonWithIcon';
import { useNavigate, useLocation } from 'react-router-dom';

const openContractsUiUrl = (): void => {
  window.open(constants.CONTRACTS_UI_URL, '_blank');
};

const openInkDocsUrl = (): void => {
  window.open(constants.INK_DOCS_URL, '_blank');
};

const openRepoUrl = (): void => {
  window.open(constants.REPO_URL, '_blank');
};

const mapIconColor = (size: number | null): { color: keyof Colors; shade: string } | undefined => {
  if (!size) return undefined;
  if (size <= constants.OPTIMAL_SIZE) {
    return { color: 'green', shade: '400' };
  } else if (size <= constants.ACCEPTABLE_SIZE) {
    return { color: 'blue', shade: '400' };
  } else if (size <= constants.PROBLEMATIC_SIZE) {
    return { color: 'yellow', shade: '400' };
  }
  return { color: 'red', shade: '400' };
};

export const Header = ({inkVersions}: any): ReactElement => {
  const [state, dispatch]: [State, Dispatch] = useContext(AppContext);
  const [, dispatchMessage]: [MessageState, MessageDispatch] = useContext(MessageContext);
  const navigate = useNavigate();

  const settingsOverlay = useRef<OverlayPanel>(null);
  const shareOverlay = useRef<OverlayPanel>(null);

  const hasDownloadableResult =
    state.compile.type === 'RESULT' &&
    state.compile.payload.type === 'OK' &&
    state.compile.payload.payload.type === 'SUCCESS';

  const iconColor = mapIconColor(state.contractSize);
  const tooltipContent = mapSizeInfo(state.contractSize);

  const compileVersion = (state: any, dispatch: any, dispatchMessage: any) => {
    if (state.ink_version) {
      compile(state, dispatch, dispatchMessage, state.ink_version)
    }
  }

  return (
    <div className="dark:text-primary dark:bg-primary dark:border-dark border-light border-b text-light flex max-h-16">
      <div className="w-32">
        <Logo className="h-16 w-32" data-testid="headerLogo" />
      </div>
      <div className="border-l max-h-8 mt-4 dark:border-dark border-light" />
      <div className={'flex p-3.5 w-full'}>
        <ButtonWithIcon
          label="Compile"
          Icon={CompileIcon}
          darkmode={state.darkmode}
          testId={'buttonIcon'}
          onClick={() => compileVersion(state, dispatch, dispatchMessage)}
          loading={state.compile.type === 'IN_PROGRESS'}
        />
        <ButtonWithIcon
          label="Test"
          Icon={TestingIcon}
          darkmode={state.darkmode}
          testId={'buttonIcon'}
          onClick={() => testing(state, dispatch, dispatchMessage)}
          loading={state.testing.type === 'IN_PROGRESS'}
        />
        <ButtonWithIcon
          label="Format"
          Icon={FormatIcon}
          darkmode={state.darkmode}
          testId={'buttonIcon'}
          onClick={() => format(state, dispatch, dispatchMessage)}
          loading={state.formatting.type === 'IN_PROGRESS'}
        />
        <ButtonWithIcon
          label="Download"
          Icon={DownloadIcon}
          darkmode={state.darkmode}
          testId={'buttonIcon'}
          onClick={() => handleDownload(state)}
          disabled={!hasDownloadableResult || !state.monacoUri}
          loading={state.compile.type === 'IN_PROGRESS'}
          iconColor={iconColor}
          tooltipContent={tooltipContent}
        />
        <ButtonWithIcon
          label="Share"
          Icon={ShareIcon}
          darkmode={state.darkmode}
          testId={'buttonIcon'}
          onClick={e => shareOverlay.current && shareOverlay.current.toggle(e, null)}
        />
        <ButtonWithIcon
          label="Settings"
          Icon={SettingsIcon}
          darkmode={state.darkmode}
          testId={'buttonIcon'}
          onClick={e => settingsOverlay.current && settingsOverlay.current.toggle(e, null)}
        />
        {inkVersions.length > 0 ? <>
          <select className='dark:bg-primary' onChange={(e) => {
            navigate(`/v${e.target.value}/`, { state: { version: e.target.value } });
            dispatch({
              type: 'SET_INK_VERSION',
              payload: e.target.value
            })
          }} value={state.ink_version || ''}>
            {inkVersions.map((v: any) => { 
              return <option key={v.ink_version} value={v.ink_version}>{v.name}</option>
              }
            )}
          </select></>: null}

        <div className="flex-grow" />

        <ButtonWithIcon
          label={'About ink!'}
          Icon={DocsIcon}
          darkmode={state.darkmode}
          testId={'buttonIconInkDocs'}
          onClick={() => {
            openInkDocsUrl();
          }}
        />
        <ButtonWithIcon
          label={'Deploy'}
          Icon={DeployIcon}
          darkmode={state.darkmode}
          testId={'buttonIconContractsUi'}
          onClick={() => {
            openContractsUiUrl();
          }}
        />
        <ButtonWithIcon
          label={'GitHub'}
          Icon={GithubRepoIcon}
          darkmode={state.darkmode}
          testId={'buttonIconRepo'}
          onClick={() => {
            openRepoUrl();
          }}
        />
      </div>
      <OverlayPanel ref={settingsOverlay} showCloseIcon dismissable>
        <SettingsSubmenu />
      </OverlayPanel>
      <OverlayPanel ref={shareOverlay} showCloseIcon dismissable>
        <ShareSubmenu darkmode={state.darkmode} />
      </OverlayPanel>
    </div>
  );
};

const handleDownload = (state: State) => {
  if (
    state.compile.type !== 'RESULT' ||
    state.compile.payload.type !== 'OK' ||
    state.compile.payload.payload.type !== 'SUCCESS'
  )
    return;

  const wasm = state.compile.payload.payload.payload.wasm;

  downloadBlob(wasm);
};

export const downloadBlob = (code: number[]): void => {
  const blob = new Blob([new Uint8Array(code).buffer]);

  const a = document.createElement('a');
  a.download = 'result.contract';
  a.href = URL.createObjectURL(blob);
  a.dataset.downloadurl = ['application/json', a.download, a.href].join(':');
  a.style.display = 'none';
  document.body.appendChild(a);
  a.click();
  document.body.removeChild(a);

  setTimeout(() => {
    URL.revokeObjectURL(a.href);
  }, 1500);
};
