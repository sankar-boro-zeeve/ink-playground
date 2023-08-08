import { State, Dispatch } from '../app/reducer';
import { MessageAction, MessageDispatch, GistMessage } from '../messages/reducer';
import * as monaco from 'monaco-editor/esm/vs/editor/editor.api';
import { GistCreateApiResponse, gistCreateRequest } from '@paritytech/ink-editor/api/gists';
import Common from '@paritytech/commontypes';
import { GIST_CREATE_URL } from '~/env';

const resetToNotAsked = (dispatch: Dispatch, dispatchMessage: MessageDispatch): void => {
  dispatch({ type: 'SET_GIST_STATE', payload: { type: 'NOT_ASKED' } });
  dispatchMessage({
    type: 'LOG_GIST',
    payload: {
      content: 'Something went wrong, please try again!',
      status: 'ERROR',
    },
  });
};

const getMessageAction = (result: GistCreateApiResponse): GistMessage => {
  switch (result.type) {
    case 'NETWORK_ERROR':
      return {
        type: 'LOG_GIST',
        payload: {
          content: 'Network Error',
          status: 'ERROR',
        },
      };
    case 'SERVER_ERROR':
      return {
        type: 'LOG_GIST',
        payload: {
          content: `Server Error: ${result.payload.status}`,
          status: 'ERROR',
        },
      };
    case 'OK':
      return handleOk(result.payload);
  }
};

const handleOk = (response: Common.GistCreateResponse): GistMessage => {
  switch (response.type) {
    case 'ERROR':
      return {
        type: 'LOG_GIST',
        payload: {
          content: `Gist creation failed: ${response.payload}`,
          status: 'ERROR',
        },
      };
    case 'SUCCESS':
      return {
        type: 'LOG_GIST',
        payload: {
          content: `Your GitHub Gist was successfully created: ${response.payload.url}`,
          status: 'DONE',
        },
      };
  }
};

export async function gistCreate(
  state: State,
  dispatch: Dispatch,
  dispatchMessage: MessageDispatch
) {
  if (state.gist.type === 'IN_PROGRESS') return;

  dispatch({ type: 'SET_GIST_STATE', payload: { type: 'IN_PROGRESS' } });

  dispatchMessage({
    type: 'LOG_GIST',
    payload: {
      content: 'Creating a new GitHub Gist...',
      status: 'IN_PROGRESS',
    },
  });

  const { monacoUri: uri } = state;

  if (!uri) {
    resetToNotAsked(dispatch, dispatchMessage);
    return;
  }

  const model = monaco.editor.getModel(uri);

  if (!model) {
    resetToNotAsked(dispatch, dispatchMessage);
    return;
  }

  const code = model.getValue();

  const result = await gistCreateRequest(
    {
      gistCreateUrl: GIST_CREATE_URL || '',
    },
    { code }
  );

  dispatch({
    type: 'SET_GIST_STATE',
    payload: { type: 'RESULT', payload: result },
  });

  const action: MessageAction | undefined = getMessageAction(result);
  if (action) dispatchMessage(action);
}
