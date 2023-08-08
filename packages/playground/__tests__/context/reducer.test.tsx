import Common from '@paritytech/commontypes';
import { reducer, defaultState, CompileState, Action } from '~/context/app/reducer';

describe('Given the reducer is used to manage state', () => {
  describe('When dark mode is toggled', () => {
    test('When dark mode is activated', () => {
      // Given, When
      const activated = reducer(defaultState, { type: 'SET_DARKMODE', payload: true });
      // Then ...
      expect(activated.darkmode).toBeTruthy();
    });

    test('When dark mode is deactivated', () => {
      // Given, When
      const deactivated = reducer(defaultState, { type: 'SET_DARKMODE', payload: false });
      // Then ...
      expect(deactivated.darkmode).toBeFalsy();
    });
  });

  describe('When numbering is toggled', () => {
    test('When numbering is activated', () => {
      // Given, When
      const activated = reducer(defaultState, { type: 'SET_NUMBERING', payload: true });
      // Then ...
      expect(activated.numbering).toBeTruthy();
    });

    test('When numbering is deactivated', () => {
      // Given, When
      const deactivated = reducer(defaultState, { type: 'SET_NUMBERING', payload: false });
      // Then ...
      expect(deactivated.numbering).toBeFalsy();
    });
  });

  describe('When minimap is toggled', () => {
    test('When minimap is activated', () => {
      // Given, When
      const activated = reducer(defaultState, { type: 'SET_MINIMAP', payload: true });
      // Then ...
      expect(activated.minimap).toBeTruthy();
    });

    test('When minimap is deactivated', () => {
      // Given, When
      const deactivated = reducer(defaultState, { type: 'SET_MINIMAP', payload: false });
      // Then ...
      expect(deactivated.minimap).toBeFalsy();
    });
  });

  test('When compile state is set to "IN_PROGRESS"', () => {
    // Given
    const type = 'SET_COMPILE_STATE';
    const payload: CompileState = {
      type: 'IN_PROGRESS',
    };
    const action: Action = {
      type,
      payload,
    };
    // When
    const resultingState = reducer(defaultState, action);
    // Then ...
    expect(resultingState.compile).toStrictEqual(payload);
  });

  test('When endpoint returns "NETWORK_ERROR"', () => {
    // Given
    const type = 'SET_COMPILE_STATE';
    const payload: CompileState = {
      type: 'RESULT',
      payload: {
        type: 'NETWORK_ERROR',
      },
    };
    const action: Action = {
      type,
      payload,
    };
    // When
    const resultingState = reducer(defaultState, action);
    // Then ...
    expect(resultingState.compile).toStrictEqual(payload);
  });

  test('When endpoint returns "SERVER_ERROR"', () => {
    // Given
    const type = 'SET_COMPILE_STATE';
    const payload: CompileState = {
      type: 'RESULT',
      payload: {
        type: 'SERVER_ERROR',
        payload: {
          status: 404,
        },
      },
    };
    const action: Action = {
      type,
      payload,
    };
    // When
    const resultingState = reducer(defaultState, action);
    // Then ...
    expect(resultingState.compile).toStrictEqual(payload);
  });

  test('When endpoint returns "OK" with "ERROR"', () => {
    // Given
    const type = 'SET_COMPILE_STATE';
    const compilationPayload: Common.CompilationResult = {
      type: 'ERROR',
      payload: {
        stdout: '',
        stderr: 'Compile failed',
      },
    };
    const payload: CompileState = {
      type: 'RESULT',
      payload: {
        type: 'OK',
        payload: compilationPayload,
      },
    };
    const action: Action = {
      type,
      payload,
    };
    // When
    const resultingState = reducer(defaultState, action);
    // Then ...
    expect(resultingState.compile).toStrictEqual(payload);
  });

  test('When endpoint returns "OK" with "SUCCESS"', () => {
    // Given
    const type = 'SET_COMPILE_STATE';
    const compilationPayload: Common.CompilationResult = {
      type: 'SUCCESS',
      payload: {
        wasm: [1, 2, 3],
        stdout: 'Compile result',
        stderr: '',
      },
    };
    const payload: CompileState = {
      type: 'RESULT',
      payload: {
        type: 'OK',
        payload: compilationPayload,
      },
    };
    const action: Action = {
      type,
      payload,
    };
    // When
    const resultingState = reducer(defaultState, action);
    // Then ...
    expect(resultingState.compile).toStrictEqual(payload);
  });
});
