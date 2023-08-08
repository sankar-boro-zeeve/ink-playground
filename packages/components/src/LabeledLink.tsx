import { ReactElement, useState } from 'react';

export type LabeledLinkProps = {
  label: string;
  link: string;
  isPlaceholderText: boolean;
};

export const LabeledLink = ({ label, link, isPlaceholderText }: LabeledLinkProps): ReactElement => {
  const [animatePing, setAnimatePing] = useState(false);
  const abbreviatedLink = link.substr(0, 32);

  const showAnimation = (): void => {
    setAnimatePing(true);
    setTimeout(() => {
      setAnimatePing(false);
    }, 750);
  };

  return (
    <div className="dark:bg-elevation dark:border-dark border-light border-t last:rounded-b pt-2 pb-4 px-4 w-80 text-lg">
      <p className="text-sm mt-1 mb-2 whitespace-nowrap">{label}</p>
      <div className="dark:bg-primary dark:border-dark bg-gray-200 border-light last:rounded-b py-2 px-4 w-full text-lg flex justify-between items-center rounded">
        {!isPlaceholderText ? (
          <>
            <a href={link} target="_blank" className="text-sm whitespace-nowrap">
              {abbreviatedLink}...
            </a>
            <button
              onClick={() => {
                showAnimation();
                // Maybe we could use an extra lib for that
                navigator.clipboard.writeText(link);
              }}
            >
              <i className={`pi pi-copy ml-4 ${animatePing ? 'animate-ping' : ''}`} />
            </button>
          </>
        ) : (
          <>
            <p className="text-sm whitespace-nowrap opacity-60">{link}</p>
            <button className="cursor-not-allowed" disabled={true}>
              <i className="pi pi-copy ml-4 opacity-60" />
            </button>
          </>
        )}
      </div>
    </div>
  );
};
