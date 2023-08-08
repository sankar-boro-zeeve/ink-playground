import Common from '@paritytech/commontypes';

export type FormattingApiRequest = Common.FormattingRequest;

export type FormattingApiResponse =
  | {
      type: 'OK';
      payload: Common.FormattingResult;
    }
  | {
      type: 'NETWORK_ERROR';
    }
  | {
      type: 'SERVER_ERROR';
      payload: { status: number };
    }
  | {
      type: 'FORMATTING_ERROR';
    };

export type Config = {
  compileUrl: string;
};

const mapResponse = async (response: Response): Promise<FormattingApiResponse> =>
  response.status === 200
    ? {
        type: 'OK',
        payload: await response.json(),
      }
    : {
        type: 'SERVER_ERROR',
        payload: { status: response.status },
      };

export const formattingRequest = (
  config: Config,
  request: FormattingApiRequest
): Promise<FormattingApiResponse> => {
  const opts: RequestInit = {
    method: 'POST',
    mode: 'cors',
    credentials: 'same-origin',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify(request),
  };

  return fetch(config.compileUrl || '', opts)
    .then(mapResponse)
    .catch(() => ({ type: 'NETWORK_ERROR' }));
};
